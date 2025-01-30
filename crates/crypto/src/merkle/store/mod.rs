#[cfg(feature = "serde")]
mod serde;

use alloc::{collections::BTreeMap, vec::Vec};
use core::borrow::Borrow;

use super::{
	EmptySubtreeRoots, InnerNode, InnerNodeInfo, MerkleError, MerklePath, MerkleTree, NodeIndex,
	RootPath, SimpleSmt, ValuePath, mmr::Mmr, node,
};
use crate::{
	hash::rpo::{Rpo256, RpoDigest},
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
		collections::{KvMap, RecordingMap},
	},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StoreNode {
	left: RpoDigest,
	right: RpoDigest,
}

impl Deserializable for StoreNode {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let left = RpoDigest::read_from(source)?;
		let right = RpoDigest::read_from(source)?;
		Ok(Self { left, right })
	}
}

impl Serializable for StoreNode {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.left.write_into(target);
		self.right.write_into(target);
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct MerkleStore<T = BTreeMap<RpoDigest, StoreNode>>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	nodes: T,
}

impl<T> MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	#[must_use]
	pub fn new() -> Self {
		let nodes = empty_hashes().collect();
		Self { nodes }
	}

	pub fn num_internal_nodes(&self) -> usize {
		self.nodes.len()
	}

	pub fn get_node(&self, root: RpoDigest, index: NodeIndex) -> Result<RpoDigest, MerkleError> {
		let mut hash = root;

		self.nodes
			.get(&hash)
			.ok_or(MerkleError::RootNotInStore(hash))?;

		for i in (0..index.depth()).rev() {
			let node = self
				.nodes
				.get(&hash)
				.ok_or(MerkleError::NodeIndexNotFoundInStore(hash, index))?;

			let bit = (index.value() >> i) & 1;
			hash = if matches!(bit, 0) {
				node.left
			} else {
				node.right
			};
		}

		Ok(hash)
	}

	pub fn get_path(&self, root: RpoDigest, index: NodeIndex) -> Result<ValuePath, MerkleError> {
		let mut hash = root;
		let mut path = Vec::with_capacity(index.depth().into());

		self.nodes
			.get(&hash)
			.ok_or(MerkleError::RootNotInStore(hash))?;

		for i in (0..index.depth()).rev() {
			let node = self
				.nodes
				.get(&hash)
				.ok_or(MerkleError::NodeIndexNotFoundInStore(hash, index))?;

			let bit = (index.value() >> i) & 1;
			hash = if matches!(bit, 0) {
				path.push(node.right);
				node.left
			} else {
				path.push(node.left);
				node.right
			}
		}

		path.reverse();

		Ok(ValuePath::new(hash, MerklePath::new(path)))
	}

	pub fn get_leaf_depth(
		&self,
		root: RpoDigest,
		tree_depth: u8,
		index: u64,
	) -> Result<u8, MerkleError> {
		if tree_depth > 64 {
			return Err(MerkleError::DepthTooBig(u64::from(tree_depth)));
		}

		NodeIndex::new(tree_depth, index)?;

		let empty = EmptySubtreeRoots::empty_hashes(tree_depth);
		let mut hash = root;
		if !self.nodes.contains_key(&hash) {
			return Err(MerkleError::RootNotInStore(hash));
		}

		let mut path = (index << (64 - tree_depth)).reverse_bits();

		for depth in 0..=tree_depth {
			if hash == empty[depth as usize] {
				return Ok(depth);
			}

			let Some(children) = self.nodes.get(&hash) else {
				return Ok(depth);
			};

			hash = if matches!(path & 1, 0) {
				children.left
			} else {
				children.right
			};
			path >>= 1;
		}

		Err(MerkleError::DepthTooBig(u64::from(tree_depth) + 1))
	}

	pub fn find_lone_leaf(
		&self,
		root: RpoDigest,
		root_index: NodeIndex,
		tree_depth: u8,
	) -> Result<Option<(NodeIndex, RpoDigest)>, MerkleError> {
		const MAX_DEPTH: u8 = u64::BITS as u8;
		if tree_depth > MAX_DEPTH {
			return Err(MerkleError::DepthTooBig(u64::from(tree_depth)));
		}

		let empty = EmptySubtreeRoots::empty_hashes(MAX_DEPTH);

		let mut node = root;
		if !self.nodes.contains_key(&node) {
			return Err(MerkleError::RootNotInStore(node));
		}

		let mut index = root_index;
		if index.depth() > tree_depth {
			return Err(MerkleError::DepthTooBig(u64::from(index.depth())));
		}

		for depth in index.depth()..tree_depth {
			let Some(children) = self.nodes.get(&node) else {
				return Ok(Some((index, node)));
			};

			let empty_node = empty[depth as usize + 1];
			node = if children.left != empty_node && children.right == empty_node {
				index = index.left_child();
				children.left
			} else if children.left == empty_node && children.right != empty_node {
				index = index.right_child();
				children.right
			} else {
				return Ok(None);
			}
		}

		if self.nodes.contains_key(&node) {
			Err(MerkleError::DepthTooBig(u64::from(tree_depth) + 1))
		} else {
			Ok(Some((index, node)))
		}
	}

	#[must_use]
	pub fn subset<R>(&self, roots: impl IntoIterator<Item = R>) -> Self
	where
		R: Borrow<RpoDigest>,
	{
		let mut store = Self::new();
		for root in roots {
			let root = *root.borrow();
			store.clone_tree_from(root, self);
		}

		store
	}

	pub fn inner_nodes(&self) -> impl Iterator<Item = InnerNodeInfo> + '_ {
		self.nodes.iter().map(|(r, n)| InnerNodeInfo {
			value: *r,
			left: n.left,
			right: n.right,
		})
	}

	pub fn non_empty_leaves(
		&self,
		root: RpoDigest,
		max_depth: u8,
	) -> impl Iterator<Item = (NodeIndex, RpoDigest)> + '_ {
		let empty_roots = EmptySubtreeRoots::empty_hashes(max_depth);
		let mut stack = Vec::new();
		stack.push((NodeIndex::new_unchecked(0, 0), root));

		core::iter::from_fn(move || {
			while let Some((index, node_hash)) = stack.pop() {
				if index.depth() == max_depth {
					return Some((index, node_hash));
				}

				if let Some(node) = self.nodes.get(&node_hash) {
					if !empty_roots.contains(&node.left) {
						stack.push((index.left_child(), node.left));
					}

					if !empty_roots.contains(&node.right) {
						stack.push((index.right_child(), node.right));
					}
				} else {
					return Some((index, node_hash));
				}
			}

			None
		})
	}

	pub fn add_merkle_path(
		&mut self,
		index: u64,
		node: RpoDigest,
		path: MerklePath,
	) -> Result<RpoDigest, MerkleError> {
		let root = path
			.inner_nodes(index, node)?
			.fold(RpoDigest::default(), |_, node| {
				let value = node.value;
				let left = node.left;
				let right = node.right;

				debug_assert_eq!(Rpo256::merge(&[left, right]), value);
				self.nodes.insert(value, StoreNode { left, right });

				node.value
			});

		Ok(root)
	}

	pub fn add_merkle_paths(
		&mut self,
		paths: impl IntoIterator<Item = (u64, RpoDigest, MerklePath)>,
	) -> Result<(), MerkleError> {
		for (index_value, node, path) in paths {
			self.add_merkle_path(index_value, node, path)?;
		}

		Ok(())
	}

	pub fn set_node(
		&mut self,
		mut root: RpoDigest,
		index: NodeIndex,
		value: RpoDigest,
	) -> Result<RootPath, MerkleError> {
		let node = value;
		let ValuePath { value, path } = self.get_path(root, index)?;

		if node != value {
			root = self.add_merkle_path(index.value(), node, path.clone())?;
		}

		Ok(RootPath { root, path })
	}

	pub fn merge_roots(
		&mut self,
		left_root: RpoDigest,
		right_root: RpoDigest,
	) -> Result<RpoDigest, MerkleError> {
		let parent = Rpo256::merge(&[left_root, right_root]);
		self.nodes.insert(parent, StoreNode {
			left: left_root,
			right: right_root,
		});

		Ok(parent)
	}

	pub fn into_inner(self) -> T {
		self.nodes
	}

	fn clone_tree_from(&mut self, root: RpoDigest, source: &Self) {
		if let Some(node) = source.nodes.get(&root) {
			if self.nodes.insert(root, *node).is_none() {
				self.clone_tree_from(node.left, source);
				self.clone_tree_from(node.right, source);
			}
		}
	}
}

impl<T> Default for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Deserializable for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let len = source.read_u64()?;
		let mut nodes = Vec::with_capacity(len as usize);

		for _ in 0..len {
			let key = RpoDigest::read_from(source)?;
			let value = StoreNode::read_from(source)?;
			nodes.push((key, value));
		}

		Ok(nodes.into_iter().collect())
	}
}

impl<T> Extend<InnerNodeInfo> for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = InnerNodeInfo>,
	{
		self.nodes.extend(iter.into_iter().map(|info| {
			(info.value, StoreNode {
				left: info.left,
				right: info.right,
			})
		}));
	}
}

impl<T> From<&MerkleTree> for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn from(value: &MerkleTree) -> Self {
		let nodes = combine_nodes_with_empty_hashes(value.inner_nodes()).collect();
		Self { nodes }
	}
}

impl<T, const DEPTH: u8> From<&SimpleSmt<DEPTH>> for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn from(value: &SimpleSmt<DEPTH>) -> Self {
		let nodes = combine_nodes_with_empty_hashes(value.inner_nodes()).collect();
		Self { nodes }
	}
}

impl<T> FromIterator<InnerNodeInfo> for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = InnerNodeInfo>,
	{
		let nodes = combine_nodes_with_empty_hashes(iter).collect();
		Self { nodes }
	}
}

impl<T> FromIterator<(RpoDigest, StoreNode)> for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = (RpoDigest, StoreNode)>,
	{
		let nodes = iter.into_iter().chain(empty_hashes()).collect();
		Self { nodes }
	}
}

impl<T> Serializable for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u64(self.nodes.len() as u64);

		for (k, v) in self.nodes.iter() {
			k.write_into(target);
			v.write_into(target);
		}
	}
}

fn empty_hashes() -> impl Iterator<Item = (RpoDigest, StoreNode)> {
	let subtrees = EmptySubtreeRoots::empty_hashes(255);
	subtrees
		.iter()
		.rev()
		.copied()
		.zip(subtrees.iter().rev().skip(1).copied())
		.map(|(child, parent)| {
			(parent, StoreNode {
				left: child,
				right: child,
			})
		})
}

fn combine_nodes_with_empty_hashes(
	nodes: impl IntoIterator<Item = InnerNodeInfo>,
) -> impl Iterator<Item = (RpoDigest, StoreNode)> {
	nodes
		.into_iter()
		.map(|info| {
			(info.value, StoreNode {
				left: info.left,
				right: info.right,
			})
		})
		.chain(empty_hashes())
}

pub type DefaultMerkleStore = MerkleStore<BTreeMap<RpoDigest, StoreNode>>;

pub type RecordingMerkleStore = MerkleStore<RecordingMap<RpoDigest, StoreNode>>;

#[cfg(test)]
mod tests {
	use assert_matches::assert_matches;
	use seq_macro::seq;
	#[cfg(feature = "std")]
	use {
		super::{Deserializable, Serializable},
		alloc::boxed::Box,
		std::error::Error,
	};

	use super::{
		DefaultMerkleStore as MerkleStore, EmptySubtreeRoots, MerkleError, MerklePath, NodeIndex,
		Rpo256, RpoDigest,
	};
	use crate::{
		Felt, ONE, WORD_SIZE, Word, ZERO,
		merkle::{
			LeafIndex, MerkleTree, PartialMerkleTree, SMT_MAX_DEPTH, SimpleSmt, digests_to_words,
			int_to_leaf, int_to_node,
		},
	};

	const KEYS_4: [u64; 4] = [0, 1, 2, 3];
	const VALUES_4: [RpoDigest; 4] = [
		int_to_node(1),
		int_to_node(2),
		int_to_node(3),
		int_to_node(4),
	];

	const KEYS_8: [u64; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
	const VALUES_8: [RpoDigest; 8] = [
		int_to_node(1),
		int_to_node(2),
		int_to_node(3),
		int_to_node(4),
		int_to_node(5),
		int_to_node(6),
		int_to_node(7),
		int_to_node(8),
	];

	#[test]
	fn root_not_in_store() -> Result<(), MerkleError> {
		let mtree = MerkleTree::new(digests_to_words(&VALUES_4))?;
		let store = MerkleStore::from(&mtree);
		assert_matches!(store.get_node(VALUES_4[0], NodeIndex::new(mtree.depth(), 0)?), Err(MerkleError::RootNotInStore(root)) if root == VALUES_4[0], "leaf 0 is not a root");

		assert_matches!(store.get_path(VALUES_4[0], NodeIndex::new(mtree.depth(), 0)?), Err(MerkleError::RootNotInStore(root)) if root == VALUES_4[0], "leaf 0 is not a root");

		Ok(())
	}

	#[test]
	fn merkle_tree() -> Result<(), MerkleError> {
		let mtree = MerkleTree::new(digests_to_words(&VALUES_4))?;
		let store = MerkleStore::from(&mtree);

		assert_eq!(
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 0)?)?,
			VALUES_4[0],
			"node 0 must be in the tree"
		);

		assert_eq!(
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 1)?)?,
			VALUES_4[1],
			"node 1 must be in the tree"
		);

		assert_eq!(
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 2)?)?,
			VALUES_4[2],
			"node 2 must be in the tree"
		);

		assert_eq!(
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 3)?)?,
			VALUES_4[3],
			"node 3 must be in the tree"
		);

		assert_eq!(
			mtree.get_node(NodeIndex::new(mtree.depth(), 0)?)?,
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 0)?)?,
			"node 0 must be the same for both MerkleTree and MerkleStore"
		);

		assert_eq!(
			mtree.get_node(NodeIndex::new(mtree.depth(), 1)?)?,
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 1)?)?,
			"node 1 must be the same for both MerkleTree and MerkleStore"
		);

		assert_eq!(
			mtree.get_node(NodeIndex::new(mtree.depth(), 2)?)?,
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 2)?)?,
			"node 2 must be the same for both MerkleTree and MerkleStore"
		);

		assert_eq!(
			mtree.get_node(NodeIndex::new(mtree.depth(), 3)?)?,
			store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 3)?)?,
			"node 3 must be the same for both MerkleTree and MerkleStore"
		);

		let result = store.get_path(mtree.root(), NodeIndex::new(mtree.depth(), 0)?)?;
		assert_eq!(
			result.value, VALUES_4[0],
			"value for merkle path at index 0 must match leaf value"
		);
		assert_eq!(
			mtree.get_path(NodeIndex::new(mtree.depth(), 0)?)?,
			result.path,
			"merkle path for index 0 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(mtree.root(), NodeIndex::new(mtree.depth(), 1)?)?;
		assert_eq!(
			result.value, VALUES_4[1],
			"value for merkle path at index 1 must match leaf value"
		);
		assert_eq!(
			mtree.get_path(NodeIndex::new(mtree.depth(), 1)?)?,
			result.path,
			"merkle path for index 1 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(mtree.root(), NodeIndex::new(mtree.depth(), 2)?)?;
		assert_eq!(
			result.value, VALUES_4[2],
			"value for merkle path at index 2 must match leaf value"
		);
		assert_eq!(
			mtree.get_path(NodeIndex::new(mtree.depth(), 2)?)?,
			result.path,
			"merkle path for index 2 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(mtree.root(), NodeIndex::new(mtree.depth(), 3)?)?;
		assert_eq!(
			result.value, VALUES_4[3],
			"value for merkle path at index 3 must match leaf value"
		);
		assert_eq!(
			mtree.get_path(NodeIndex::new(mtree.depth(), 3)?)?,
			result.path,
			"merkle path for index 3 must be the same for the MerkleTree and MerkleStore"
		);

		Ok(())
	}

	#[test]
	fn empty_roots() -> Result<(), MerkleError> {
		let store = MerkleStore::default();
		let mut root = RpoDigest::default();

		for depth in 0..255 {
			root = Rpo256::merge(&[root; 2]);
			assert!(
				store.get_node(root, NodeIndex::new(0, 0)?).is_ok(),
				"the root of the empty tree of depth {depth} must be registered"
			);
		}

		Ok(())
	}

	#[test]
	fn leaf_paths_for_empty_trees() -> Result<(), MerkleError> {
		let store = MerkleStore::default();

		seq!(DEPTH in 1u8..64u8 {
			let smt = SimpleSmt::<DEPTH>::new()?;

			let index = NodeIndex::new(DEPTH, 0)?;
			let store_path = store.get_path(smt.root(), index)?;
			let smt_path = smt.open(&LeafIndex::new(0)?).path;
			assert_eq!(store_path.value, RpoDigest::default(), "the leaf of an empty tree is always zero");
			assert_eq!(store_path.path, smt_path, "the returned merkle path does not match the computed values");
			assert_eq!(
				store_path.path.compute_root(DEPTH.into(), RpoDigest::default())?,
				smt.root(),
				"computed root from the path must match the empty tree root"
			);
		});

		Ok(())
	}

	#[test]
	fn get_invalid_node() -> Result<(), MerkleError> {
		let mtree = MerkleTree::new(digests_to_words(&VALUES_4))?;
		let store = MerkleStore::from(&mtree);
		let _ = store.get_node(mtree.root(), NodeIndex::new(mtree.depth(), 3)?);

		Ok(())
	}

	#[test]
	fn add_sparse_merkle_tree_one_level() -> Result<(), MerkleError> {
		let keys2 = [0u64, 1];
		let leaves2 = [int_to_leaf(1), int_to_leaf(2)];
		let smt = SimpleSmt::<1>::with_leaves(keys2.into_iter().zip(leaves2))?;
		let store = MerkleStore::from(&smt);

		let idx = NodeIndex::new(1, 0)?;
		assert_eq!(smt.get_node(idx)?, leaves2[0].into());
		assert_eq!(store.get_node(smt.root(), idx)?, smt.get_node(idx)?);

		let idx = NodeIndex::new(1, 1)?;
		assert_eq!(smt.get_node(idx)?, leaves2[1].into());
		assert_eq!(store.get_node(smt.root(), idx)?, smt.get_node(idx)?);

		Ok(())
	}

	#[test]
	fn sparse_merkle_tree() -> Result<(), MerkleError> {
		let smt = SimpleSmt::<SMT_MAX_DEPTH>::with_leaves(
			KEYS_4.into_iter().zip(digests_to_words(&VALUES_4)),
		)?;

		let store = MerkleStore::from(&smt);

		assert_eq!(
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 0)?)?,
			VALUES_4[0],
			"node 0 must be in the tree"
		);

		assert_eq!(
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 1)?)?,
			VALUES_4[1],
			"node 1 must be in the tree"
		);

		assert_eq!(
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 2)?)?,
			VALUES_4[2],
			"node 2 must be in the tree"
		);

		assert_eq!(
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 3)?)?,
			VALUES_4[3],
			"node 3 must be in the tree"
		);

		assert_eq!(
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 4)?)?,
			RpoDigest::default(),
			"unmodified node 4 must be zero"
		);

		assert_eq!(
			smt.get_node(NodeIndex::new(SMT_MAX_DEPTH, 0)?)?,
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 0)?)?,
			"node 0 must be the same for both the MerkleTree and MerkleStore"
		);

		assert_eq!(
			smt.get_node(NodeIndex::new(SMT_MAX_DEPTH, 1)?)?,
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 1)?)?,
			"node 1 must be the same for both the MerkleTree and MerkleStore"
		);

		assert_eq!(
			smt.get_node(NodeIndex::new(SMT_MAX_DEPTH, 2)?)?,
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 2)?)?,
			"node 2 must be the same for both the MerkleTree and MerkleStore"
		);

		assert_eq!(
			smt.get_node(NodeIndex::new(SMT_MAX_DEPTH, 3)?)?,
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 3)?)?,
			"node 3 must be the same for both the MerkleTree and MerkleStore"
		);

		assert_eq!(
			smt.get_node(NodeIndex::new(SMT_MAX_DEPTH, 4)?)?,
			store.get_node(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 4)?)?,
			"node 4 must be the same for both the MerkleTree and MerkleStore"
		);

		let result = store.get_path(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 0)?)?;
		assert_eq!(
			result.value, VALUES_4[0],
			"value for merkle path at index 0 must match leaf value"
		);
		assert_eq!(
			result.path,
			smt.open(&LeafIndex::new(0)?).path,
			"merkle path for index 0 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 1)?)?;
		assert_eq!(
			result.value, VALUES_4[1],
			"value for merkle path at index 1 must match leaf value"
		);
		assert_eq!(
			result.path,
			smt.open(&LeafIndex::new(1)?).path,
			"merkle path for index 1 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 2)?)?;
		assert_eq!(
			result.value, VALUES_4[2],
			"value for merkle path at index 2 must match leaf value"
		);
		assert_eq!(
			result.path,
			smt.open(&LeafIndex::new(2)?).path,
			"merkle path for index 2 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 3)?)?;
		assert_eq!(
			result.value, VALUES_4[3],
			"value for merkle path at index 3 must match leaf value"
		);
		assert_eq!(
			result.path,
			smt.open(&LeafIndex::new(3)?).path,
			"merkle path for index 3 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(smt.root(), NodeIndex::new(SMT_MAX_DEPTH, 4)?)?;
		assert_eq!(
			result.value,
			RpoDigest::default(),
			"value for merkle path at index 4 must match leaf value"
		);
		assert_eq!(
			result.path,
			smt.open(&LeafIndex::new(4)?).path,
			"merkle path for index 4 must be the same for the MerkleTree and MerkleStore"
		);

		Ok(())
	}

	#[test]
	fn add_merkle_path() -> Result<(), MerkleError> {
		let mtree = MerkleTree::new(digests_to_words(&VALUES_4))?;

		let i0 = 0;
		let p0 = mtree.get_path(NodeIndex::new(2, i0)?)?;

		let i1 = 1;
		let p1 = mtree.get_path(NodeIndex::new(2, i1)?)?;

		let i2 = 2;
		let p2 = mtree.get_path(NodeIndex::new(2, i2)?)?;

		let i3 = 3;
		let p3 = mtree.get_path(NodeIndex::new(2, i3)?)?;

		let paths = [
			(i0, VALUES_4[i0 as usize], p0),
			(i1, VALUES_4[i1 as usize], p1),
			(i2, VALUES_4[i2 as usize], p2),
			(i3, VALUES_4[i3 as usize], p3),
		];

		let mut store = MerkleStore::default();
		store.add_merkle_paths(paths.clone())?;

		let pmt = PartialMerkleTree::with_paths(paths)?;

		assert_eq!(
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 0)?)?,
			VALUES_4[0],
			"node 0 must be in the pmt"
		);

		assert_eq!(
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 1)?)?,
			VALUES_4[1],
			"node 1 must be in the pmt"
		);

		assert_eq!(
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 2)?)?,
			VALUES_4[2],
			"node 2 must be in the pmt"
		);

		assert_eq!(
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 3)?)?,
			VALUES_4[3],
			"node 3 must be in the pmt"
		);

		assert_eq!(
			pmt.get_node(NodeIndex::new(pmt.max_depth(), 0)?)?,
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 0)?)?,
			"node 0 must be the same for both PartialMerkleTree and MerkleStore"
		);

		assert_eq!(
			pmt.get_node(NodeIndex::new(pmt.max_depth(), 1)?)?,
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 1)?)?,
			"node 1 must be the same for both PartialMerkleTree and MerkleStore"
		);

		assert_eq!(
			pmt.get_node(NodeIndex::new(pmt.max_depth(), 2)?)?,
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 2)?)?,
			"node 2 must be the same for both PartialMerkleTree and MerkleStore"
		);

		assert_eq!(
			pmt.get_node(NodeIndex::new(pmt.max_depth(), 3)?)?,
			store.get_node(pmt.root(), NodeIndex::new(pmt.max_depth(), 3)?)?,
			"node 3 must be the same for both PartialMerkleTree and MerkleStore"
		);

		let result = store.get_path(pmt.root(), NodeIndex::new(pmt.max_depth(), 0)?)?;
		assert_eq!(
			result.value, VALUES_4[0],
			"value for merkle path at index 0 must match leaf value"
		);
		assert_eq!(
			result.path,
			pmt.get_path(NodeIndex::new(pmt.max_depth(), 0)?)?,
			"merkle path for index 0 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(pmt.root(), NodeIndex::new(pmt.max_depth(), 1)?)?;
		assert_eq!(
			result.value, VALUES_4[1],
			"value for merkle path at index 1 must match leaf value"
		);
		assert_eq!(
			result.path,
			pmt.get_path(NodeIndex::new(pmt.max_depth(), 1)?)?,
			"merkle path for index 1 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(pmt.root(), NodeIndex::new(pmt.max_depth(), 2)?)?;
		assert_eq!(
			result.value, VALUES_4[2],
			"value for merkle path at index 2 must match leaf value"
		);
		assert_eq!(
			result.path,
			pmt.get_path(NodeIndex::new(pmt.max_depth(), 2)?)?,
			"merkle path for index 2 must be the same for the MerkleTree and MerkleStore"
		);

		let result = store.get_path(pmt.root(), NodeIndex::new(pmt.max_depth(), 3)?)?;
		assert_eq!(
			result.value, VALUES_4[3],
			"value for merkle path at index 3 must match leaf value"
		);
		assert_eq!(
			result.path,
			pmt.get_path(NodeIndex::new(pmt.max_depth(), 3)?)?,
			"merkle path for index 3 must be the same for the MerkleTree and MerkleStore"
		);

		Ok(())
	}

	#[test]
	fn wont_open_to_different_depth_root() -> Result<(), MerkleError> {
		let empty = EmptySubtreeRoots::empty_hashes(64);
		let a = [ONE; 4];
		let b = [Felt::new(2); 4];

		let mut root = Rpo256::merge(&[a.into(), b.into()]);
		for depth in (1..=63).rev() {
			root = Rpo256::merge(&[root, empty[depth]]);
		}

		let mtree = MerkleTree::new([a, b])?;
		let store = MerkleStore::from(&mtree);
		let index = NodeIndex::root();
		let err = store.get_node(root, index).err().unwrap();

		assert_matches!(err, MerkleError::RootNotInStore(err_root) if err_root == root);

		Ok(())
	}

	#[allow(clippy::many_single_char_names)]
	#[test]
	fn store_path_opens_from_leaf() -> Result<(), MerkleError> {
		let a = [ONE; 4];
		let b = [Felt::new(2); 4];
		let c = [Felt::new(3); 4];
		let d = [Felt::new(4); 4];
		let e = [Felt::new(5); 4];
		let f = [Felt::new(6); 4];
		let g = [Felt::new(7); 4];
		let h = [Felt::new(8); 4];

		let i = Rpo256::merge(&[a.into(), b.into()]);
		let j = Rpo256::merge(&[c.into(), d.into()]);
		let k = Rpo256::merge(&[e.into(), f.into()]);
		let l = Rpo256::merge(&[g.into(), h.into()]);

		let m = Rpo256::merge(&[i, j]);
		let n = Rpo256::merge(&[k, l]);

		let root = Rpo256::merge(&[m, n]);

		let mtree = MerkleTree::new([a, b, c, d, e, f, g, h])?;
		let store = MerkleStore::from(&mtree);
		let path = store.get_path(root, NodeIndex::new(3, 1)?)?.path;

		let expected = MerklePath::new([a.into(), j, n].to_vec());
		assert_eq!(path, expected);

		Ok(())
	}

	#[test]
	fn set_node() -> Result<(), MerkleError> {
		let mtree = MerkleTree::new(digests_to_words(&VALUES_4))?;
		let mut store = MerkleStore::from(&mtree);
		let value = int_to_node(42);
		let index = NodeIndex::new(mtree.depth(), 0)?;
		let new_root = store.set_node(mtree.root(), index, value)?.root;
		assert_eq!(
			store.get_node(new_root, index)?,
			value,
			"value must have changed"
		);

		Ok(())
	}

	#[test]
	fn constructors() -> Result<(), MerkleError> {
		const DEPTH: u8 = 32;

		let mtree = MerkleTree::new(digests_to_words(&VALUES_4))?;
		let store = MerkleStore::from(&mtree);

		let depth = mtree.depth();
		let leaves = 2u64.pow(depth.into());
		for index in 0..leaves {
			let index = NodeIndex::new(depth, index)?;
			let value_path = store.get_path(mtree.root(), index)?;
			assert_eq!(mtree.get_path(index)?, value_path.path);
		}
		let smt =
			SimpleSmt::<DEPTH>::with_leaves(KEYS_4.into_iter().zip(digests_to_words(&VALUES_4)))?;
		let store = MerkleStore::from(&smt);

		for key in KEYS_4 {
			let index = NodeIndex::new(DEPTH, key)?;
			let value_path = store.get_path(smt.root(), index)?;
			assert_eq!(smt.open(&LeafIndex::new(key)?).path, value_path.path);
		}

		let d = 2;
		let paths = [
			(0, VALUES_4[0], mtree.get_path(NodeIndex::new(d, 0)?)?),
			(1, VALUES_4[1], mtree.get_path(NodeIndex::new(d, 1)?)?),
			(2, VALUES_4[2], mtree.get_path(NodeIndex::new(d, 2)?)?),
			(3, VALUES_4[3], mtree.get_path(NodeIndex::new(d, 3)?)?),
		];

		let mut store1 = MerkleStore::default();
		store1.add_merkle_paths(paths.clone())?;

		let mut store2 = MerkleStore::default();
		store2.add_merkle_path(0, VALUES_4[0], mtree.get_path(NodeIndex::new(d, 0)?)?)?;
		store2.add_merkle_path(1, VALUES_4[1], mtree.get_path(NodeIndex::new(d, 1)?)?)?;
		store2.add_merkle_path(2, VALUES_4[2], mtree.get_path(NodeIndex::new(d, 2)?)?)?;
		store2.add_merkle_path(3, VALUES_4[3], mtree.get_path(NodeIndex::new(d, 3)?)?)?;
		let pmt = PartialMerkleTree::with_paths(paths)?;

		for key in [0, 1, 2, 3] {
			let index = NodeIndex::new(d, key)?;
			let value_path1 = store1.get_path(pmt.root(), index)?;
			let value_path2 = store2.get_path(pmt.root(), index)?;
			assert_eq!(value_path1, value_path2);

			assert_eq!(pmt.get_path(index)?, value_path1.path);
		}

		Ok(())
	}

	#[test]
	#[allow(clippy::unreadable_literal)]
	fn node_path_should_be_truncated_by_midtier_insert() -> Result<(), MerkleError> {
		let key = 0b11010010_11001100_11001100_11001100_11001100_11001100_11001100_11001100_u64;

		let mut store = MerkleStore::new();
		let root = EmptySubtreeRoots::empty_hashes(64)[0];

		let depth = 64;
		let node = RpoDigest::from([Felt::new(key); WORD_SIZE]);
		let index = NodeIndex::new(depth, key)?;
		let root = store.set_node(root, index, node)?.root;
		let result = store.get_node(root, index)?;
		let path = store.get_path(root, index)?.path;
		assert_eq!(node, result);
		assert_eq!(path.depth(), depth);
		assert!(path.verify(index.value(), result, root).is_ok());

		let key = key ^ (1 << 63);
		let key = key >> 8;
		let depth = 56;
		let node = RpoDigest::from([Felt::new(key); WORD_SIZE]);
		let index = NodeIndex::new(depth, key)?;
		let root = store.set_node(root, index, node)?.root;
		let result = store.get_node(root, index)?;
		let path = store.get_path(root, index)?.path;
		assert_eq!(node, result);
		assert_eq!(path.depth(), depth);
		assert!(path.verify(index.value(), result, root).is_ok());

		let key = key << 8;
		let index = NodeIndex::new(64, key)?;
		assert!(store.get_node(root, index).is_err());

		Ok(())
	}

	#[test]
	fn get_leaf_depth_works_depth_64() -> Result<(), MerkleError> {
		let mut store = MerkleStore::new();
		let mut root = EmptySubtreeRoots::empty_hashes(64)[0];
		let key = u64::MAX;

		for d in 0..64 {
			let k = key & (u64::MAX >> d);
			let node = RpoDigest::from([Felt::new(k); WORD_SIZE]);
			let index = NodeIndex::new(64, k)?;

			assert_eq!(d, store.get_leaf_depth(root, 64, k)?);

			root = store.set_node(root, index, node)?.root;
			assert_eq!(store.get_leaf_depth(root, 64, k)?, 64);
		}

		Ok(())
	}
}
