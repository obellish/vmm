use alloc::{
	collections::{BTreeMap, VecDeque},
	vec::Vec,
};

use crate::{
	crypto::hash::RpoDigest,
	mast::{MastForest, MastForestError, MastNode, MastNodeId},
};

type ForestIndex = usize;

pub(crate) struct MultiMastForestNodeIterator<'forest> {
	mast_forests: Vec<&'forest MastForest>,
	current_forest_idx: ForestIndex,
	current_procedure_root_idx: u32,
	non_external_nodes: BTreeMap<RpoDigest, (ForestIndex, MastNodeId)>,
	discovered_nodes: Vec<Vec<bool>>,
	unvisited_nodes: VecDeque<MultiMastForestIteratorItem>,
}

impl<'forest> MultiMastForestNodeIterator<'forest> {
	pub(crate) fn new(mast_forests: Vec<&'forest MastForest>) -> Self {
		let discovered_nodes = mast_forests
			.iter()
			.map(|forest| vec![false; forest.num_nodes() as usize])
			.collect();

		let mut non_external_nodes = BTreeMap::new();

		for (forest_idx, forest) in mast_forests.iter().enumerate() {
			for (node_idx, node) in forest.nodes().iter().enumerate() {
				let node_id = MastNodeId::new_unchecked(node_idx as u32);
				if !node.is_external() {
					non_external_nodes.insert(node.digest(), (forest_idx, node_id));
				}
			}
		}

		Self {
			mast_forests,
			current_forest_idx: 0,
			current_procedure_root_idx: 0,
			non_external_nodes,
			discovered_nodes,
			unvisited_nodes: VecDeque::new(),
		}
	}

	fn push_node(&mut self, forest_idx: ForestIndex, node_id: MastNodeId) {
		self.unvisited_nodes
			.push_back(MultiMastForestIteratorItem::Node {
				forest_idx,
				node_id,
			});
		self.discovered_nodes[forest_idx][node_id.as_usize()] = true;
	}

	fn discover_tree(
		&mut self,
		forest_idx: ForestIndex,
		node_id: MastNodeId,
	) -> Result<(), MastForestError> {
		if self.discovered_nodes[forest_idx][node_id.as_usize()] {
			return Ok(());
		}

		let current_node = &self.mast_forests[forest_idx]
			.nodes()
			.get(node_id.as_usize())
			.ok_or_else(|| {
				MastForestError::NodeIdOverflow(
					node_id,
					self.mast_forests[forest_idx].num_nodes() as usize,
				)
			})?;

		match current_node {
			MastNode::Join(node) => {
				self.discover_tree(forest_idx, node.first())?;
				self.discover_tree(forest_idx, node.second())?;
				self.push_node(forest_idx, node_id);
			}
			MastNode::Split(node) => {
				self.discover_tree(forest_idx, node.on_true())?;
				self.discover_tree(forest_idx, node.on_false())?;
				self.push_node(forest_idx, node_id);
			}
			MastNode::Loop(node) => {
				self.discover_tree(forest_idx, node.body())?;
				self.push_node(forest_idx, node_id);
			}
			MastNode::Call(node) => {
				self.discover_tree(forest_idx, node.callee())?;
				self.push_node(forest_idx, node_id);
			}
			MastNode::Dyn(_) | MastNode::Block(_) => self.push_node(forest_idx, node_id),
			MastNode::External(node) => {
				if let Some((other_forest_idx, other_node_id)) =
					self.non_external_nodes.get(&node.digest()).copied()
				{
					self.discover_tree(other_forest_idx, other_node_id)?;

					self.unvisited_nodes.push_back(
						MultiMastForestIteratorItem::ExternalNodeReplacement {
							replacement_forest_idx: other_forest_idx,
							replacement_node_id: other_node_id,
							replaced_forest_idx: forest_idx,
							replaced_node_id: node_id,
						},
					);

					self.discovered_nodes[forest_idx][node_id.as_usize()] = true;
				} else {
					self.push_node(forest_idx, node_id);
				}
			}
		}

		Ok(())
	}

	fn discover_nodes(&mut self) {
		'forest_loop: while self.current_forest_idx < self.mast_forests.len()
			&& self.unvisited_nodes.is_empty()
		{
			if self.mast_forests.is_empty() {
				return;
			}

			if matches!(
				self.mast_forests[self.current_forest_idx].num_procedures(),
				0
			) {
				self.current_forest_idx += 1;
				continue;
			}

			let procedure_roots = self.mast_forests[self.current_forest_idx].procedure_roots();
			let discovered_nodes = &self.discovered_nodes[self.current_forest_idx];

			while discovered_nodes
				[procedure_roots[self.current_procedure_root_idx as usize].as_usize()]
			{
				if self.current_procedure_root_idx + 1
					>= self.mast_forests[self.current_forest_idx].num_procedures()
				{
					self.current_procedure_root_idx = 0;
					self.current_forest_idx += 1;

					continue 'forest_loop;
				}

				self.current_procedure_root_idx += 1;
			}

			let procedure_root_id = procedure_roots[self.current_procedure_root_idx as usize];
			self.discover_tree(self.current_forest_idx, procedure_root_id)
				.expect("we should only pass root indices that are valid for the forest");
		}
	}
}

impl Iterator for MultiMastForestNodeIterator<'_> {
	type Item = MultiMastForestIteratorItem;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(deque_item) = self.unvisited_nodes.pop_front() {
			return Some(deque_item);
		}

		self.discover_nodes();

		if self.unvisited_nodes.is_empty() {
			None
		} else {
			self.next()
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultiMastForestIteratorItem {
	Node {
		forest_idx: ForestIndex,
		node_id: MastNodeId,
	},
	ExternalNodeReplacement {
		replacement_forest_idx: ForestIndex,
		replacement_node_id: MastNodeId,
		replaced_forest_idx: ForestIndex,
		replaced_node_id: MastNodeId,
	},
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use super::{MultiMastForestIteratorItem, MultiMastForestNodeIterator};
	use crate::{
		Operation,
		crypto::hash::RpoDigest,
		mast::{MastForest, MastForestError, MastNode, MastNodeId},
	};

	fn random_digest() -> RpoDigest {
		RpoDigest::new([
			rand_utils::rand_value(),
			rand_utils::rand_value(),
			rand_utils::rand_value(),
			rand_utils::rand_value(),
		])
	}

	#[test]
	fn multi_mast_forest_dfs_empty() {
		let forest = MastForest::new();
		let mut iterator = MultiMastForestNodeIterator::new(vec![&forest]);
		assert!(iterator.next().is_none());
	}

	#[test]
	fn multi_mast_forest_multiple_forests_dfs() -> Result<(), MastForestError> {
		let nodea0_digest = random_digest();
		let nodea1_digest = random_digest();
		let nodea2_digest = random_digest();
		let nodea3_digest = random_digest();

		let nodeb0_digest = random_digest();

		let mut forest_a = MastForest::new();
		forest_a.add_external(nodea0_digest)?;
		let id1 = forest_a.add_external(nodea1_digest)?;
		let id2 = forest_a.add_external(nodea2_digest)?;
		let id3 = forest_a.add_external(nodea3_digest)?;
		let id_split = forest_a.add_split(id2, id3)?;
		let id_join = forest_a.add_join(id2, id_split)?;

		forest_a.make_root(id_join);
		forest_a.make_root(id1);

		let mut forest_b = MastForest::new();
		let id_ext_b = forest_b.add_external(nodeb0_digest)?;
		let id_block_b = forest_b.add_block(vec![Operation::Eqz], None)?;
		let id_split_b = forest_b.add_split(id_ext_b, id_block_b)?;

		forest_b.make_root(id_split_b);

		let nodes =
			MultiMastForestNodeIterator::new(vec![&forest_a, &forest_b]).collect::<Vec<_>>();

		assert_eq!(nodes.len(), 8);
		assert_eq!(nodes[0], MultiMastForestIteratorItem::Node {
			forest_idx: 0,
			node_id: id2
		});
		assert_eq!(nodes[1], MultiMastForestIteratorItem::Node {
			forest_idx: 0,
			node_id: id3
		});
		assert_eq!(nodes[2], MultiMastForestIteratorItem::Node {
			forest_idx: 0,
			node_id: id_split
		});
		assert_eq!(nodes[3], MultiMastForestIteratorItem::Node {
			forest_idx: 0,
			node_id: id_join
		});
		assert_eq!(nodes[4], MultiMastForestIteratorItem::Node {
			forest_idx: 0,
			node_id: id1
		});
		assert_eq!(nodes[5], MultiMastForestIteratorItem::Node {
			forest_idx: 1,
			node_id: id_ext_b
		});
		assert_eq!(nodes[6], MultiMastForestIteratorItem::Node {
			forest_idx: 1,
			node_id: id_block_b
		});
		assert_eq!(nodes[7], MultiMastForestIteratorItem::Node {
			forest_idx: 1,
			node_id: id_split_b
		});

		Ok(())
	}

	#[test]
	fn multi_mast_forest_external_dependencies() -> Result<(), MastForestError> {
		let block_foo = MastNode::basic_block(vec![Operation::Drop], None)?;
		let mut forest_a = MastForest::new();
		let id_foo_a = forest_a.add_external(block_foo.digest())?;
		let id_call_a = forest_a.add_call(id_foo_a)?;
		forest_a.make_root(id_call_a);

		let mut forest_b = MastForest::new();
		let id_ext_b = forest_b.add_external(forest_a[id_call_a].digest())?;
		let id_call_b = forest_b.add_call(id_ext_b)?;
		forest_b.add_node(block_foo)?;
		forest_b.make_root(id_call_b);

		let nodes =
			MultiMastForestNodeIterator::new(vec![&forest_a, &forest_b]).collect::<Vec<_>>();

		assert_eq!(nodes.len(), 5);

		assert_eq!(nodes[0], MultiMastForestIteratorItem::Node {
			forest_idx: 1,
			node_id: MastNodeId::new_unchecked(2)
		});
		assert_eq!(
			nodes[1],
			MultiMastForestIteratorItem::ExternalNodeReplacement {
				replacement_forest_idx: 1,
				replacement_node_id: MastNodeId::new_unchecked(2),
				replaced_forest_idx: 0,
				replaced_node_id: MastNodeId::new_unchecked(0)
			}
		);
		assert_eq!(nodes[2], MultiMastForestIteratorItem::Node {
			forest_idx: 0,
			node_id: MastNodeId::new_unchecked(1)
		});
		assert_eq!(
			nodes[3],
			MultiMastForestIteratorItem::ExternalNodeReplacement {
				replacement_forest_idx: 0,
				replacement_node_id: MastNodeId::new_unchecked(1),
				replaced_forest_idx: 1,
				replaced_node_id: MastNodeId::new_unchecked(0)
			}
		);
		assert_eq!(nodes[4], MultiMastForestIteratorItem::Node {
			forest_idx: 1,
			node_id: MastNodeId::new_unchecked(1)
		});

		Ok(())
	}

	#[test]
	fn multi_mast_forest_child_duplicate() -> Result<(), MastForestError> {
		let block_foo = MastNode::basic_block(vec![Operation::Drop], None)?;
		let mut forest = MastForest::new();
		let id_foo = forest.add_external(block_foo.digest())?;
		let id_call1 = forest.add_call(id_foo)?;
		let id_call2 = forest.add_call(id_foo)?;
		let id_split = forest.add_split(id_call1, id_call2)?;
		forest.make_root(id_split);

		let nodes = MultiMastForestNodeIterator::new(vec![&forest]).collect::<Vec<_>>();

		for (i, expected_node_id) in [id_foo, id_call1, id_call2, id_split]
			.into_iter()
			.enumerate()
		{
			assert_eq!(nodes[i], MultiMastForestIteratorItem::Node {
				forest_idx: 0,
				node_id: expected_node_id
			});
		}

		Ok(())
	}
}
