use alloc::{collections::BTreeMap, vec::Vec};

use crate::{
	crypto::hash::Blake3Digest,
	mast::{
		DecoratorId, MastForest, MastForestError, MastNode, MastNodeFingerprint, MastNodeId,
		MultiMastForestIteratorItem, MultiMastForestNodeIterator,
	},
	utils::collections::KvMap,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct MastForestRootMap {
	root_maps: Vec<BTreeMap<MastNodeId, MastNodeId>>,
}

impl MastForestRootMap {
	fn from_node_id_map(id_map: Vec<MastForestNodeIdMap>, forests: Vec<&MastForest>) -> Self {
		let mut root_maps = vec![BTreeMap::new(); forests.len()];

		for (forest_idx, forest) in forests.into_iter().enumerate() {
			for root in forest.procedure_roots() {
				let new_id = id_map[forest_idx]
					.get(root)
					.copied()
					.expect("every node id should be mapped to its new id");
				root_maps[forest_idx].insert(*root, new_id);
			}
		}

		Self { root_maps }
	}

	#[must_use]
	pub fn map_root(&self, forest_index: usize, root: MastNodeId) -> Option<MastNodeId> {
		self.root_maps
			.get(forest_index)
			.and_then(|map| map.get(&root))
			.copied()
	}
}

pub(crate) struct MastForestMerger {
	mast_forest: MastForest,
	node_id_by_hash: BTreeMap<MastNodeFingerprint, MastNodeId>,
	hash_by_node_id: BTreeMap<MastNodeId, MastNodeFingerprint>,
	decorators_by_hash: BTreeMap<Blake3Digest<32>, DecoratorId>,
	decorator_id_mappings: Vec<DecoratorIdMap>,
	node_id_mappings: Vec<MastForestNodeIdMap>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl MastForestMerger {
	pub(crate) fn merge<'forest>(
		forests: impl IntoIterator<Item = &'forest MastForest>,
	) -> Result<(MastForest, MastForestRootMap), MastForestError> {
		let forests = forests.into_iter().collect::<Vec<_>>();
		let decorator_id_mappings = Vec::with_capacity(forests.len());
		let node_id_mappings = vec![MastForestNodeIdMap::new(); forests.len()];

		let mut merger = Self {
			node_id_by_hash: BTreeMap::new(),
			hash_by_node_id: BTreeMap::new(),
			decorators_by_hash: BTreeMap::new(),
			mast_forest: MastForest::new(),
			decorator_id_mappings,
			node_id_mappings,
		};

		merger.merge_inner(forests.clone())?;

		let Self {
			mast_forest,
			node_id_mappings,
			..
		} = merger;

		let root_maps = MastForestRootMap::from_node_id_map(node_id_mappings, forests);

		Ok((mast_forest, root_maps))
	}

	fn merge_inner(&mut self, forests: Vec<&MastForest>) -> Result<(), MastForestError> {
		for other_forest in &forests {
			self.merge_advice_map(other_forest)?;
		}

		for other_forest in &forests {
			self.merge_decorators(other_forest)?;
		}

		let iterator = MultiMastForestNodeIterator::new(forests.clone());
		for item in iterator {
			match item {
				MultiMastForestIteratorItem::Node {
					forest_idx,
					node_id,
				} => {
					let node = &forests[forest_idx][node_id];
					self.merge_node(forest_idx, node_id, node)?;
				}
				MultiMastForestIteratorItem::ExternalNodeReplacement {
					replacement_forest_idx,
					replacement_node_id,
					replaced_forest_idx,
					replaced_node_id,
				} => {
					let mapped_replacement = self.node_id_mappings[replacement_forest_idx]
						.get(&replacement_node_id)
						.copied()
						.expect("every merged node id should be mapped");

					self.node_id_mappings[replaced_forest_idx]
						.insert(replaced_node_id, mapped_replacement);
				}
			}
		}

		for (forest_idx, forest) in forests.iter().enumerate() {
			self.merge_roots(forest_idx, forest)?;
		}

		Ok(())
	}

	fn merge_decorators(&mut self, other_forest: &MastForest) -> Result<(), MastForestError> {
		let mut decorator_id_remapping = DecoratorIdMap::new(other_forest.decorators().len());

		for (merging_id, merging_decorator) in other_forest.decorators().iter().enumerate() {
			let merging_decorator_hash = merging_decorator.fingerprint();
			let new_decorator_id = if let Some(existing_decorator) =
				self.decorators_by_hash.get(&merging_decorator_hash)
			{
				*existing_decorator
			} else {
				let new_decorator_id = self.mast_forest.add_decorator(merging_decorator.clone())?;
				self.decorators_by_hash
					.insert(merging_decorator_hash, new_decorator_id);
				new_decorator_id
			};

			decorator_id_remapping.insert(
				DecoratorId::new_unchecked(merging_id as u32),
				new_decorator_id,
			);
		}

		self.decorator_id_mappings.push(decorator_id_remapping);

		Ok(())
	}

	fn merge_advice_map(&mut self, other_forest: &MastForest) -> Result<(), MastForestError> {
		for (digest, values) in other_forest.advice_map().iter() {
			if let Some(stored_values) = self.mast_forest.advice_map().get(digest) {
				if stored_values != values {
					return Err(MastForestError::AdviceMapKeyCollisionOnMerge(*digest));
				}
			} else {
				self.mast_forest
					.advice_map_mut()
					.insert(*digest, values.clone());
			}
		}

		Ok(())
	}

	fn merge_node(
		&mut self,
		forest_idx: usize,
		merging_id: MastNodeId,
		node: &MastNode,
	) -> Result<(), MastForestError> {
		let remapped_node = self.remap_node(forest_idx, node)?;

		let node_fingerprint = MastNodeFingerprint::from_mast_node(
			&self.mast_forest,
			&self.hash_by_node_id,
			&remapped_node,
		)?;

		if let Some(matching_node_id) = self.lookup_node_by_fingerprint(&node_fingerprint) {
			self.node_id_mappings[forest_idx].insert(merging_id, matching_node_id);
		} else {
			let new_node_id = self.mast_forest.add_node(remapped_node)?;
			self.node_id_mappings[forest_idx].insert(merging_id, new_node_id);

			self.node_id_by_hash.insert(node_fingerprint, new_node_id);
			self.hash_by_node_id.insert(new_node_id, node_fingerprint);
		}

		Ok(())
	}

	fn merge_roots(
		&mut self,
		forest_idx: usize,
		other_forest: &MastForest,
	) -> Result<(), MastForestError> {
		for root_id in other_forest.procedure_roots() {
			let new_root = self.node_id_mappings[forest_idx]
				.get(root_id)
				.expect("all node ids should have an entry");

			self.mast_forest.make_root(*new_root);
		}

		Ok(())
	}

	fn remap_node(&self, forest_idx: usize, node: &MastNode) -> Result<MastNode, MastForestError> {
		let map_decorator_id = |decorator_id: &DecoratorId| {
			self.decorator_id_mappings[forest_idx]
				.get(*decorator_id)
				.ok_or_else(|| {
					MastForestError::DecoratorIdOverflow(
						*decorator_id,
						self.decorator_id_mappings[forest_idx].len(),
					)
				})
		};
		let map_decorators = |decorators: &[DecoratorId]| -> Result<Vec<_>, MastForestError> {
			decorators.iter().map(map_decorator_id).collect()
		};

		let map_node_id = |node_id: MastNodeId| {
			self.node_id_mappings[forest_idx]
				.get(&node_id)
				.copied()
				.expect("every node id should have an entry")
		};

		let mut mapped_node = match node {
			MastNode::Join(node) => {
				let first = map_node_id(node.first());
				let second = map_node_id(node.second());

				MastNode::join(first, second, &self.mast_forest)?
			}
			MastNode::Split(node) => {
				let if_branch = map_node_id(node.on_true());
				let else_branch = map_node_id(node.on_false());

				MastNode::split(if_branch, else_branch, &self.mast_forest)?
			}
			MastNode::Loop(node) => {
				let body = map_node_id(node.body());
				MastNode::r#loop(body, &self.mast_forest)?
			}
			MastNode::Call(node) => {
				let callee = map_node_id(node.callee());
				MastNode::call(callee, &self.mast_forest)?
			}
			MastNode::Block(node) => MastNode::basic_block(
				node.operations().copied().collect(),
				Some(
					node.decorators()
						.iter()
						.map(|(idx, decorator_id)| match map_decorator_id(decorator_id) {
							Ok(mapped_decorator) => Ok((*idx, mapped_decorator)),
							Err(err) => Err(err),
						})
						.collect::<Result<Vec<_>, _>>()?,
				),
			)?,
			MastNode::Dyn(_) => MastNode::r#dyn(),
			MastNode::External(node) => MastNode::external(node.digest()),
		};

		if !mapped_node.is_basic_block() {
			mapped_node.set_before_enter(map_decorators(node.before_enter())?);
			mapped_node.set_after_exit(map_decorators(node.after_exit())?);
		}

		Ok(mapped_node)
	}

	fn lookup_node_by_fingerprint(&self, fingerprint: &MastNodeFingerprint) -> Option<MastNodeId> {
		self.node_id_by_hash.get(fingerprint).copied()
	}
}

#[repr(transparent)]
struct DecoratorIdMap {
	inner: Vec<Option<DecoratorId>>,
}

impl DecoratorIdMap {
	fn new(num_ids: usize) -> Self {
		Self {
			inner: vec![None; num_ids],
		}
	}

	fn insert(&mut self, key: DecoratorId, value: DecoratorId) {
		self.inner[key.as_usize()] = Some(value);
	}

	fn get(&self, key: DecoratorId) -> Option<DecoratorId> {
		self.inner.get(key.as_usize()).and_then(|o| *o)
	}

	fn len(&self) -> usize {
		self.inner.len()
	}
}

type MastForestNodeIdMap = BTreeMap<MastNodeId, MastNodeId>;
