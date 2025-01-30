#[cfg(feature = "serde")]
mod serde;

use alloc::{
	collections::{BTreeMap, BTreeSet},
	string::String,
	vec::Vec,
};
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use super::{InnerNodeInfo, MerkleError, MerklePath, NodeIndex, ValuePath};
use crate::{
	EMPTY_WORD, Word,
	hash::rpo::{Rpo256, RpoDigest},
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, word_to_hex,
	},
};

const ROOT_INDEX: NodeIndex = NodeIndex::root();

const EMPTY_DIGEST: RpoDigest = RpoDigest::new(EMPTY_WORD);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialMerkleTree {
	max_depth: u8,
	nodes: BTreeMap<NodeIndex, RpoDigest>,
	leaves: BTreeSet<NodeIndex>,
}

impl PartialMerkleTree {
	pub const MAX_DEPTH: u8 = 64;
	pub const MIN_DEPTH: u8 = 1;

	#[must_use]
	pub const fn new() -> Self {
		Self {
			max_depth: 0,
			nodes: BTreeMap::new(),
			leaves: BTreeSet::new(),
		}
	}

	pub fn with_paths(
		paths: impl IntoIterator<Item = (u64, RpoDigest, MerklePath)>,
	) -> Result<Self, MerkleError> {
		let tree = Self::new();

		paths
			.into_iter()
			.try_fold(tree, |mut tree, (index, value, path)| {
				tree.add_path(index, value, path)?;
				Ok(tree)
			})
	}

	pub fn with_leaves<I>(entries: impl IntoIterator<IntoIter = I>) -> Result<Self, MerkleError>
	where
		I: ExactSizeIterator + Iterator<Item = (NodeIndex, RpoDigest)>,
	{
		let mut layers = BTreeMap::<u8, Vec<u64>>::new();
		let mut leaves = BTreeSet::new();
		let mut nodes = BTreeMap::new();

		for (node_index, hash) in entries {
			leaves.insert(node_index);
			nodes.insert(node_index, hash);
			layers
				.entry(node_index.depth())
				.and_modify(|layer_vec| layer_vec.push(node_index.value()))
				.or_insert_with(|| vec![node_index.value()]);
		}

		let max = 2usize.pow(63);
		if layers.len() > max {
			return Err(MerkleError::TooManyEntries(max));
		}

		let max_depth = *layers.keys().next_back().unwrap_or(&0);

		for depth in 0..max_depth {
			layers.entry(depth).or_default();
		}

		let mut layer_iter = layers.into_values().rev();
		let mut parent_layer = layer_iter.next().unwrap();
		let mut current_layer;

		for depth in (1..max_depth + 1).rev() {
			current_layer = layer_iter.next().unwrap();
			core::mem::swap(&mut current_layer, &mut parent_layer);

			for index_value in current_layer {
				let parent_node = NodeIndex::new(depth - 1, index_value / 2)?;

				if !parent_layer.contains(&parent_node.value()) {
					let index = NodeIndex::new(depth, index_value)?;

					let node = nodes
						.get(&index)
						.ok_or(MerkleError::NodeIndexNotFoundInTree(index))?;

					let sibling = nodes
						.get(&index.sibling())
						.ok_or_else(|| MerkleError::NodeIndexNotFoundInTree(index.sibling()))?;

					let parent = Rpo256::merge(&index.build_node(*node, *sibling));

					parent_layer.push(parent_node.value());

					nodes.insert(parent_node, parent);
				}
			}
		}

		Ok(Self {
			max_depth,
			nodes,
			leaves,
		})
	}

	#[must_use]
	pub fn root(&self) -> RpoDigest {
		self.nodes.get(&ROOT_INDEX).copied().unwrap_or(EMPTY_DIGEST)
	}

	#[must_use]
	pub const fn max_depth(&self) -> u8 {
		self.max_depth
	}

	pub fn get_node(&self, index: NodeIndex) -> Result<RpoDigest, MerkleError> {
		self.nodes
			.get(&index)
			.ok_or(MerkleError::NodeIndexNotFoundInTree(index))
			.copied()
	}

	#[must_use]
	pub fn is_leaf(&self, index: NodeIndex) -> bool {
		self.leaves.contains(&index)
	}

	#[must_use]
	pub fn to_paths(&self) -> Vec<(NodeIndex, ValuePath)> {
		let mut paths = Vec::new();
		self.leaves.iter().for_each(|&leaf| {
			paths.push((leaf, ValuePath {
				value: self.get_node(leaf).expect("failed to get leaf node"),
				path: self.get_path(leaf).expect("failed to get path"),
			}));
		});

		paths
	}

	pub fn get_path(&self, mut index: NodeIndex) -> Result<MerklePath, MerkleError> {
		if index.is_root() {
			return Err(MerkleError::DepthTooSmall(index.depth()));
		} else if index.depth() > self.max_depth() {
			return Err(MerkleError::DepthTooBig(index.depth().into()));
		}

		if !self.nodes.contains_key(&index) {
			return Err(MerkleError::NodeIndexNotFoundInTree(index));
		}

		let mut path = Vec::new();
		for _ in 0..index.depth() {
			let sibling_index = index.sibling();
			index.move_up();
			let sibling = self
				.nodes
				.get(&sibling_index)
				.copied()
				.expect("sibling node not in the map");
			path.push(sibling);
		}

		Ok(MerklePath::new(path))
	}

	pub fn leaves(&self) -> impl Iterator<Item = (NodeIndex, RpoDigest)> + '_ {
		self.leaves.iter().map(|&leaf| {
			(
				leaf,
				self.get_node(leaf)
					.unwrap_or_else(|_| panic!("leaf with {leaf} is not in the nodes map")),
			)
		})
	}

	pub fn inner_nodes(&self) -> impl Iterator<Item = InnerNodeInfo> + '_ {
		let inner_nodes = self
			.nodes
			.iter()
			.filter(|(index, _)| !self.leaves.contains(index));

		inner_nodes.map(|(index, digest)| {
			let left_hash = self
				.nodes
				.get(&index.left_child())
				.expect("failed to get left child hash");

			let right_hash = self
				.nodes
				.get(&index.right_child())
				.expect("failed to get right child hash");

			InnerNodeInfo {
				value: *digest,
				left: *left_hash,
				right: *right_hash,
			}
		})
	}

	pub fn add_path(
		&mut self,
		index_value: u64,
		value: RpoDigest,
		path: MerklePath,
	) -> Result<(), MerkleError> {
		let index_value = NodeIndex::new(path.len() as u8, index_value)?;

		Self::check_depth(index_value.depth())?;
		self.update_depth(index_value.depth());

		self.leaves.insert(index_value);
		let sibling_node_index = index_value.sibling();
		self.leaves.insert(sibling_node_index);

		self.nodes.insert(index_value, value);
		self.nodes.insert(sibling_node_index, path[0]);

		let mut index_value = index_value;
		let node = Rpo256::merge(&index_value.build_node(value, path[0]));
		let root = path.iter().skip(1).copied().fold(node, |node, hash| {
			index_value.move_up();
			self.nodes.insert(index_value, node);

			self.leaves.remove(&index_value);

			let sibling_node = index_value.sibling();

			if self.nodes.insert(sibling_node, hash).is_none() {
				self.leaves.insert(sibling_node);
			}

			Rpo256::merge(&index_value.build_node(node, hash))
		});

		if self.root() == EMPTY_DIGEST {
			self.nodes.insert(ROOT_INDEX, root);
		} else if self.root() != root {
			return Err(MerkleError::ConflictingRoots {
				expected: self.root(),
				actual: root,
			});
		}

		Ok(())
	}

	pub fn update_leaf(&mut self, index: u64, value: Word) -> Result<RpoDigest, MerkleError> {
		let mut node_index = NodeIndex::new(self.max_depth(), index)?;

		for _ in 0..node_index.depth() {
			if !self.leaves.contains(&node_index) {
				node_index.move_up();
			}
		}

		let old_value = self
			.nodes
			.insert(node_index, value.into())
			.ok_or(MerkleError::NodeIndexNotFoundInTree(node_index))?;

		if value == *old_value {
			return Ok(old_value);
		}

		let mut value = value.into();
		for _ in 0..node_index.depth() {
			let sibling = self
				.nodes
				.get(&node_index.sibling())
				.expect("sibling should exist");
			value = Rpo256::merge(&node_index.build_node(value, *sibling));
			node_index.move_up();
			self.nodes.insert(node_index, value);
		}

		Ok(old_value)
	}

	fn update_depth(&mut self, new_depth: u8) {
		self.max_depth = new_depth.max(self.max_depth);
	}

	fn check_depth(depth: u8) -> Result<(), MerkleError> {
		if depth < Self::MIN_DEPTH {
			Err(MerkleError::DepthTooSmall(depth))
		} else if Self::MAX_DEPTH < depth {
			Err(MerkleError::DepthTooBig(depth.into()))
		} else {
			Ok(())
		}
	}
}

impl Default for PartialMerkleTree {
	fn default() -> Self {
		Self::new()
	}
}

impl Deserializable for PartialMerkleTree {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let leaves_len = source.read_u64()? as usize;
		let mut leaf_nodes = Vec::with_capacity(leaves_len);

		for _ in 0..leaves_len {
			let index = NodeIndex::read_from(source)?;
			let hash = RpoDigest::read_from(source)?;
			leaf_nodes.push((index, hash));
		}

		let pmt = Self::with_leaves(leaf_nodes).map_err(|_| {
			DeserializationError::InvalidValue(String::from(
				"invalid data for PartialMerkleTree creation",
			))
		})?;

		Ok(pmt)
	}
}

impl Serializable for PartialMerkleTree {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u64(self.leaves.len() as u64);
		for leaf_index in &self.leaves {
			leaf_index.write_into(target);
			self.get_node(*leaf_index)
				.expect("leaf hash not found")
				.write_into(target);
		}
	}
}

#[cfg(test)]
mod tests {
	use alloc::{collections::BTreeMap, vec::Vec};

	use super::PartialMerkleTree;
	use crate::{
		hash::rpo::RpoDigest,
		merkle::{
			DefaultMerkleStore as MerkleStore, InnerNodeInfo, MerkleError, MerkleTree, NodeIndex,
			ValuePath, digests_to_words, int_to_node,
		},
		utils::{Deserializable, DeserializationError, Serializable},
	};

	const NODE_10: NodeIndex = NodeIndex::new_unchecked(1, 0);
	const NODE_11: NodeIndex = NodeIndex::new_unchecked(1, 1);

	const NODE_20: NodeIndex = NodeIndex::new_unchecked(2, 0);
	const NODE_21: NodeIndex = NodeIndex::new_unchecked(2, 1);
	const NODE_22: NodeIndex = NodeIndex::new_unchecked(2, 2);
	const NODE_23: NodeIndex = NodeIndex::new_unchecked(2, 3);

	const NODE_30: NodeIndex = NodeIndex::new_unchecked(3, 0);
	const NODE_31: NodeIndex = NodeIndex::new_unchecked(3, 1);
	const NODE_32: NodeIndex = NodeIndex::new_unchecked(3, 2);
	const NODE_33: NodeIndex = NodeIndex::new_unchecked(3, 3);

	const VALUES_8: [RpoDigest; 8] = [
		int_to_node(30),
		int_to_node(31),
		int_to_node(32),
		int_to_node(33),
		int_to_node(34),
		int_to_node(35),
		int_to_node(36),
		int_to_node(37),
	];

	#[test]
	fn with_leaves() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let leave_nodes = [
			(NODE_20, mt.get_node(NODE_20)?),
			(NODE_32, mt.get_node(NODE_32)?),
			(NODE_33, mt.get_node(NODE_33)?),
			(NODE_22, mt.get_node(NODE_22)?),
			(NODE_23, mt.get_node(NODE_23)?),
		];

		let leaf_nodes: BTreeMap<NodeIndex, RpoDigest> = leave_nodes.into_iter().collect();

		let pmt = PartialMerkleTree::with_leaves(leaf_nodes)?;

		assert_eq!(pmt.root(), expected_root);

		Ok(())
	}

	#[test]
	fn err_with_leaves() {
		let leaf_nodes = [
			(NODE_20, int_to_node(20)),
			(NODE_32, int_to_node(32)),
			(NODE_33, int_to_node(33)),
			(NODE_23, int_to_node(23)),
		];

		let leaf_nodes: BTreeMap<NodeIndex, RpoDigest> = leaf_nodes.into_iter().collect();

		assert!(PartialMerkleTree::with_leaves(leaf_nodes).is_err());
	}

	#[test]
	fn get_root() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);
		let path33 = ms.get_path(expected_root, NODE_33)?;

		let pmt = PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		assert_eq!(pmt.root(), expected_root);

		Ok(())
	}

	#[test]
	fn add_and_get_paths() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let expected_path33 = ms.get_path(expected_root, NODE_33)?;
		let expected_path22 = ms.get_path(expected_root, NODE_22)?;

		let mut pmt = PartialMerkleTree::new();
		pmt.add_path(3, expected_path33.value, expected_path33.path.clone())?;
		pmt.add_path(2, expected_path22.value, expected_path22.path.clone())?;

		let path33 = pmt.get_path(NODE_33)?;
		let path22 = pmt.get_path(NODE_22)?;
		let actual_root = pmt.root();

		assert_eq!(path33, expected_path33.path);
		assert_eq!(path22, expected_path22.path);
		assert_eq!(actual_root, expected_root);

		Ok(())
	}

	#[test]
	fn get_node() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;

		let pmt = PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		assert_eq!(ms.get_node(expected_root, NODE_32)?, pmt.get_node(NODE_32)?);
		assert_eq!(ms.get_node(expected_root, NODE_10)?, pmt.get_node(NODE_10)?);

		Ok(())
	}

	#[test]
	fn update_leaf() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let root = mt.root();

		let mut ms = MerkleStore::from(&mt);
		let path33 = ms.get_path(root, NODE_33)?;

		let mut pmt =
			PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		let new_value32 = int_to_node(132);
		let expected_root = ms.set_node(root, NODE_32, new_value32)?.root;

		pmt.update_leaf(2, *new_value32)?;
		let actual_root = pmt.root();

		assert_eq!(actual_root, expected_root);

		let new_value20 = int_to_node(120);
		let expected_root = ms.set_node(expected_root, NODE_20, new_value20)?.root;

		pmt.update_leaf(0, *new_value20)?;
		let actual_root = pmt.root();

		assert_eq!(actual_root, expected_root);

		let new_value11 = int_to_node(111);
		let expected_root = ms.set_node(expected_root, NODE_11, new_value11)?.root;

		pmt.update_leaf(6, *new_value11)?;
		let actual_root = pmt.root();

		assert_eq!(actual_root, expected_root);

		Ok(())
	}

	#[test]
	fn get_paths() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;
		let path22 = ms.get_path(expected_root, NODE_22)?;

		let mut pmt = PartialMerkleTree::new();
		pmt.add_path(3, path33.value, path33.path)?;
		pmt.add_path(2, path22.value, path22.path)?;

		let leaves = [NODE_20, NODE_22, NODE_23, NODE_32, NODE_33];
		let expected_paths = leaves
			.iter()
			.map(|&leaf| {
				Ok((leaf, ValuePath {
					value: mt.get_node(leaf)?,
					path: mt.get_path(leaf)?,
				}))
			})
			.collect::<Result<Vec<(NodeIndex, ValuePath)>, MerkleError>>()?;

		let actual_paths = pmt.to_paths();

		assert_eq!(actual_paths, expected_paths);

		Ok(())
	}

	#[test]
	fn leaves() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;
		let path22 = ms.get_path(expected_root, NODE_22)?;

		let mut pmt =
			PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		let value11 = mt.get_node(NODE_11)?;
		let value20 = mt.get_node(NODE_20)?;
		let value32 = mt.get_node(NODE_32)?;
		let value33 = mt.get_node(NODE_33)?;

		let leaves = [
			(NODE_11, value11),
			(NODE_20, value20),
			(NODE_32, value32),
			(NODE_33, value33),
		];

		let expected_leaves = leaves.iter().copied();
		assert!(expected_leaves.eq(pmt.leaves()));

		pmt.add_path(2, path22.value, path22.path)?;

		let value20 = mt.get_node(NODE_20)?;
		let value22 = mt.get_node(NODE_22)?;
		let value23 = mt.get_node(NODE_23)?;
		let value33 = mt.get_node(NODE_33)?;

		let leaves = [
			(NODE_20, value20),
			(NODE_22, value22),
			(NODE_23, value23),
			(NODE_32, value32),
			(NODE_33, value33),
		];

		let expected_leaves = leaves.iter().copied();
		assert!(expected_leaves.eq(pmt.leaves()));

		Ok(())
	}

	#[test]
	fn inner_node_iterator() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;
		let path22 = ms.get_path(expected_root, NODE_22)?;

		let mut pmt =
			PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		let actual = pmt.inner_nodes().collect::<Vec<_>>();

		let expected_n00 = mt.root();
		let expected_n10 = mt.get_node(NODE_10)?;
		let expected_n11 = mt.get_node(NODE_11)?;
		let expected_n20 = mt.get_node(NODE_20)?;
		let expected_n21 = mt.get_node(NODE_21)?;
		let expected_n32 = mt.get_node(NODE_32)?;
		let expected_n33 = mt.get_node(NODE_33)?;

		let mut expected = vec![
			InnerNodeInfo {
				value: expected_n00,
				left: expected_n10,
				right: expected_n11,
			},
			InnerNodeInfo {
				value: expected_n10,
				left: expected_n20,
				right: expected_n21,
			},
			InnerNodeInfo {
				value: expected_n21,
				left: expected_n32,
				right: expected_n33,
			},
		];

		assert_eq!(actual, expected);

		pmt.add_path(2, path22.value, path22.path)?;

		let actual = pmt.inner_nodes().collect::<Vec<_>>();

		let expected_n22 = mt.get_node(NODE_22)?;
		let expected_n23 = mt.get_node(NODE_23)?;

		let info_11 = InnerNodeInfo {
			value: expected_n11,
			left: expected_n22,
			right: expected_n23,
		};

		expected.insert(2, info_11);

		assert_eq!(actual, expected);

		Ok(())
	}

	#[test]
	fn serialization() -> eyre::Result<()> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;
		let path22 = ms.get_path(expected_root, NODE_22)?;

		let pmt = PartialMerkleTree::with_paths([
			(3, path33.value, path33.path),
			(2, path22.value, path22.path),
		])?;

		let serialized_pmt = pmt.to_bytes();
		let deserialized_pmt = PartialMerkleTree::read_from_bytes(&serialized_pmt)?;

		assert_eq!(deserialized_pmt, pmt);

		Ok(())
	}

	#[test]
	fn err_deserialization() {
		let mut tree_bytes = vec![5];
		tree_bytes.append(&mut NODE_20.to_bytes());
		tree_bytes.append(&mut int_to_node(20).to_bytes());

		tree_bytes.append(&mut NODE_21.to_bytes());
		tree_bytes.append(&mut int_to_node(21).to_bytes());

		tree_bytes.append(&mut vec![1, 2]);
		tree_bytes.append(&mut int_to_node(11).to_bytes());

		assert!(PartialMerkleTree::read_from_bytes(&tree_bytes).is_err());
	}

	#[test]
	fn err_add_path() -> Result<(), MerkleError> {
		let path33 = [int_to_node(1), int_to_node(2), int_to_node(3)]
			.into_iter()
			.collect();
		let path22 = [int_to_node(4), int_to_node(5)].into_iter().collect();

		let mut pmt = PartialMerkleTree::new();
		pmt.add_path(3, int_to_node(6), path33)?;

		assert!(pmt.add_path(2, int_to_node(7), path22).is_err());

		Ok(())
	}

	#[test]
	fn err_get_node() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;

		let pmt = PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		assert!(pmt.get_node(NODE_22).is_err());
		assert!(pmt.get_node(NODE_23).is_err());
		assert!(pmt.get_node(NODE_30).is_err());
		assert!(pmt.get_node(NODE_31).is_err());

		Ok(())
	}

	#[test]
	fn err_get_path() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;

		let pmt = PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		assert!(pmt.get_path(NODE_22).is_err());
		assert!(pmt.get_path(NODE_23).is_err());
		assert!(pmt.get_path(NODE_30).is_err());
		assert!(pmt.get_path(NODE_31).is_err());

		Ok(())
	}

	#[test]
	fn err_update_leaf() -> Result<(), MerkleError> {
		let mt = MerkleTree::new(digests_to_words(&VALUES_8))?;
		let expected_root = mt.root();

		let ms = MerkleStore::from(&mt);

		let path33 = ms.get_path(expected_root, NODE_33)?;

		let mut pmt =
			PartialMerkleTree::with_paths(core::iter::once((3, path33.value, path33.path)))?;

		assert!(pmt.update_leaf(8, *int_to_node(38)).is_err());

		Ok(())
	}
}
