#[cfg(feature = "serde")]
mod serde;

use alloc::collections::{BTreeMap, BTreeSet};

use super::{InnerNode, LeafIndex, MutationSet, SMT_MAX_DEPTH, SMT_MIN_DEPTH, SparseMerkleTree};
use crate::{
	EMPTY_WORD, Word,
	hash::rpo::RpoDigest,
	merkle::{EmptySubtreeRoots, InnerNodeInfo, MerkleError, NodeIndex, ValuePath},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleSmt<const DEPTH: u8> {
	root: RpoDigest,
	leaves: BTreeMap<u64, Word>,
	inner_nodes: BTreeMap<NodeIndex, InnerNode>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl<const DEPTH: u8> SimpleSmt<DEPTH> {
	pub const fn new() -> Result<Self, MerkleError> {
		if DEPTH < SMT_MIN_DEPTH {
			return Err(MerkleError::DepthTooSmall(DEPTH));
		} else if SMT_MAX_DEPTH < DEPTH {
			return Err(MerkleError::DepthTooBig(DEPTH as u64));
		}

		let root = *EmptySubtreeRoots::entry(DEPTH, 0);

		Ok(Self {
			root,
			leaves: BTreeMap::new(),
			inner_nodes: BTreeMap::new(),
		})
	}

	pub fn with_leaves(
		entries: impl IntoIterator<Item = (u64, Word)>,
	) -> Result<Self, MerkleError> {
		let mut tree = Self::new()?;

		let max_num_entries = 2usize.pow(DEPTH.min(63).into());

		let mut key_set_to_zero = BTreeSet::new();

		for (idx, (key, value)) in entries.into_iter().enumerate() {
			if idx >= max_num_entries {
				return Err(MerkleError::TooManyEntries(max_num_entries));
			}

			let old_value = tree.insert(LeafIndex::<DEPTH>::new(key)?, value);

			if old_value != Self::EMPTY_VALUE || key_set_to_zero.contains(&key) {
				return Err(MerkleError::DuplicateValuesForIndex(key));
			}

			if value == Self::EMPTY_VALUE {
				key_set_to_zero.insert(key);
			}
		}

		Ok(tree)
	}

	pub fn with_contiguous_leaves(
		entries: impl IntoIterator<Item = Word>,
	) -> Result<Self, MerkleError> {
		Self::with_leaves(
			entries
				.into_iter()
				.enumerate()
				.map(|(idx, word)| (idx.try_into().expect("tree max depth is 2^8"), word)),
		)
	}

	#[must_use]
	#[allow(clippy::unused_self)]
	pub const fn depth(&self) -> u8 {
		DEPTH
	}

	#[must_use]
	pub fn root(&self) -> RpoDigest {
		<Self as SparseMerkleTree<DEPTH>>::root(self)
	}

	#[must_use]
	pub fn num_leaves(&self) -> usize {
		self.leaves.len()
	}

	#[must_use]
	pub fn get_leaf(&self, key: &LeafIndex<DEPTH>) -> Word {
		<Self as SparseMerkleTree<DEPTH>>::get_leaf(self, key)
	}

	pub fn get_node(&self, index: NodeIndex) -> Result<RpoDigest, MerkleError> {
		if index.is_root() {
			Err(MerkleError::DepthTooSmall(index.depth()))
		} else if index.depth() > DEPTH {
			Err(MerkleError::DepthTooBig(u64::from(index.depth())))
		} else if index.depth() == DEPTH {
			let leaf = self.get_leaf(&LeafIndex::<DEPTH>::try_from(index)?);

			Ok(leaf.into())
		} else {
			Ok(self.get_inner_node(index).hash())
		}
	}

	#[must_use]
	pub fn open(&self, key: &LeafIndex<DEPTH>) -> ValuePath {
		<Self as SparseMerkleTree<DEPTH>>::open(self, key)
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		debug_assert_eq!(self.leaves.is_empty(), self.root == Self::EMPTY_ROOT);
		self.root == Self::EMPTY_ROOT
	}

	pub fn leaves(&self) -> impl Iterator<Item = (u64, &Word)> {
		self.leaves.iter().map(|(i, w)| (*i, w))
	}

	pub fn inner_nodes(&self) -> impl Iterator<Item = InnerNodeInfo> + '_ {
		self.inner_nodes.values().map(|e| InnerNodeInfo {
			value: e.hash(),
			left: e.left,
			right: e.right,
		})
	}

	pub fn insert(&mut self, key: LeafIndex<DEPTH>, value: Word) -> Word {
		<Self as SparseMerkleTree<DEPTH>>::insert(self, key, value)
	}

	pub fn compute_mutations(
		&self,
		kv_pairs: impl IntoIterator<Item = (LeafIndex<DEPTH>, Word)>,
	) -> MutationSet<LeafIndex<DEPTH>, Word, DEPTH> {
		<Self as SparseMerkleTree<DEPTH>>::compute_mutations(self, kv_pairs)
	}

	pub fn apply_mutations(
		&mut self,
		mutations: MutationSet<LeafIndex<DEPTH>, Word, DEPTH>,
	) -> Result<(), MerkleError> {
		<Self as SparseMerkleTree<DEPTH>>::apply_mutations(self, mutations)
	}

	pub fn apply_mutations_with_reversion(
		&mut self,
		mutations: MutationSet<LeafIndex<DEPTH>, Word, DEPTH>,
	) -> Result<MutationSet<LeafIndex<DEPTH>, Word, DEPTH>, MerkleError> {
		<Self as SparseMerkleTree<DEPTH>>::apply_mutations_with_reversion(self, mutations)
	}

	pub fn set_subtree<const SUBTREE_DEPTH: u8>(
		&mut self,
		subtree_insertion_index: u64,
		subtree: SimpleSmt<SUBTREE_DEPTH>,
	) -> Result<RpoDigest, MerkleError> {
		if SUBTREE_DEPTH > DEPTH {
			return Err(MerkleError::SubtreeDepthExceedsDepth {
				subtree_depth: SUBTREE_DEPTH,
				tree_depth: DEPTH,
			});
		}

		let subtree_root_insertion_depth = DEPTH - SUBTREE_DEPTH;
		let subtree_root_index =
			NodeIndex::new(subtree_root_insertion_depth, subtree_insertion_index)?;

		let leaf_index_shift = subtree_insertion_index * 2u64.pow(SUBTREE_DEPTH.into());
		for (subtree_leaf_idx, leaf_value) in subtree.leaves() {
			let new_leaf_idx = leaf_index_shift + subtree_leaf_idx;
			debug_assert!(new_leaf_idx < 2u64.pow(DEPTH.into()));

			self.leaves.insert(new_leaf_idx, *leaf_value);
		}

		for (branch_idx, branch_node) in subtree.inner_nodes {
			let new_branch_idx = {
				let new_depth = subtree_root_insertion_depth + branch_idx.depth();
				let new_value = subtree_insertion_index * 2u64.pow(branch_idx.depth().into())
					+ branch_idx.value();

				NodeIndex::new(new_depth, new_value)?
			};

			self.inner_nodes.insert(new_branch_idx, branch_node);
		}

		self.recompute_nodes_from_index_to_root(subtree_root_index, subtree.root);

		Ok(self.root)
	}
}

impl<const DEPTH: u8> SparseMerkleTree<DEPTH> for SimpleSmt<DEPTH> {
	type Key = LeafIndex<DEPTH>;
	type Leaf = Word;
	type Opening = ValuePath;
	type Value = Word;

	const EMPTY_ROOT: RpoDigest = *EmptySubtreeRoots::entry(DEPTH, 0);
	const EMPTY_VALUE: Self::Value = EMPTY_WORD;

	fn root(&self) -> RpoDigest {
		self.root
	}

	fn set_root(&mut self, root: RpoDigest) {
		self.root = root;
	}

	fn get_inner_node(&self, index: NodeIndex) -> InnerNode {
		self.inner_nodes
			.get(&index)
			.copied()
			.unwrap_or_else(|| EmptySubtreeRoots::get_inner_node(DEPTH, index.depth()))
	}

	fn insert_inner_node(&mut self, index: NodeIndex, inner_node: InnerNode) -> Option<InnerNode> {
		self.inner_nodes.insert(index, inner_node)
	}

	fn remove_inner_node(&mut self, index: NodeIndex) -> Option<InnerNode> {
		self.inner_nodes.remove(&index)
	}

	fn insert_value(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
		if value == Self::EMPTY_VALUE {
			self.leaves.remove(&key.value())
		} else {
			self.leaves.insert(key.value(), value)
		}
	}

	fn get_value(&self, key: &Self::Key) -> Self::Value {
		self.get_leaf(key)
	}

	fn get_leaf(&self, key: &Self::Key) -> Self::Leaf {
		let leaf_pos = key.value();
		self.leaves
			.get(&leaf_pos)
			.map_or(Self::EMPTY_VALUE, |word| *word)
	}

	fn hash_leaf(leaf: &Self::Leaf) -> RpoDigest {
		leaf.into()
	}

	fn construct_prospective_leaf(
		&self,
		_: Self::Leaf,
		_: &Self::Key,
		value: &Self::Value,
	) -> Self::Leaf {
		*value
	}

	fn key_to_leaf_index(key: &Self::Key) -> LeafIndex<DEPTH> {
		*key
	}

	fn path_and_leaf_to_opening(
		path: crate::merkle::MerklePath,
		leaf: Self::Leaf,
	) -> Self::Opening {
		(path, leaf).into()
	}
}

#[allow(clippy::many_single_char_names)]
#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use assert_matches::assert_matches;

	use super::SimpleSmt;
	use crate::{
		EMPTY_WORD, Word,
		hash::rpo::{Rpo256, RpoDigest},
		merkle::{
			EmptySubtreeRoots, InnerNodeInfo, LeafIndex, MerkleError, MerkleTree, NodeIndex,
			digests_to_words, int_to_leaf, int_to_node, smt::SparseMerkleTree,
		},
	};

	const KEYS_4: [u64; 4] = [0, 1, 2, 3];
	const KEYS_8: [u64; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

	const VALUES_4: [RpoDigest; 4] = [
		int_to_node(1),
		int_to_node(2),
		int_to_node(3),
		int_to_node(4),
	];

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

	const ZERO_VALUES_8: [Word; 8] = [int_to_leaf(0); 8];

	fn compute_internal_nodes() -> (RpoDigest, RpoDigest, RpoDigest) {
		let node2 = Rpo256::merge(&[VALUES_4[0], VALUES_4[1]]);
		let node3 = Rpo256::merge(&[VALUES_4[2], VALUES_4[3]]);
		let root = Rpo256::merge(&[node2, node3]);

		(root, node2, node3)
	}

	#[test]
	fn build_empty_tree() -> Result<(), MerkleError> {
		let smt = SimpleSmt::<3>::new()?;
		let mt = MerkleTree::new(ZERO_VALUES_8)?;

		assert_eq!(mt.root(), smt.root());

		Ok(())
	}

	#[test]
	fn build_sparse_tree() -> Result<(), MerkleError> {
		const DEPTH: u8 = 3;
		let mut smt = SimpleSmt::<DEPTH>::new()?;
		let mut values = ZERO_VALUES_8.to_vec();

		assert_eq!(smt.num_leaves(), 0);

		let key = 6;
		let new_node = int_to_leaf(7);
		values[key as usize] = new_node;
		let old_value = smt.insert(LeafIndex::<DEPTH>::new(key)?, new_node);
		let mt2 = MerkleTree::new(values.clone())?;
		assert_eq!(mt2.root(), smt.root());
		assert_eq!(
			mt2.get_path(NodeIndex::new(3, 6)?)?,
			smt.open(&LeafIndex::<3>::new(6)?).path
		);
		assert_eq!(old_value, EMPTY_WORD);
		assert_eq!(smt.num_leaves(), 1);

		let key = 2;
		let new_node = int_to_leaf(3);
		values[key as usize] = new_node;
		let old_value = smt.insert(LeafIndex::new(key)?, new_node);
		let mt3 = MerkleTree::new(values)?;
		assert_eq!(mt3.root(), smt.root());
		assert_eq!(
			mt3.get_path(NodeIndex::new(3, 2)?)?,
			smt.open(&LeafIndex::new(2)?).path
		);
		assert_eq!(old_value, EMPTY_WORD);
		assert_eq!(smt.num_leaves(), 2);

		Ok(())
	}

	#[test]
	fn build_contiguous_tree() -> Result<(), MerkleError> {
		let tree_with_leaves =
			SimpleSmt::<2>::with_leaves([0, 1, 2, 3].into_iter().zip(digests_to_words(&VALUES_4)))?;

		let tree_with_contiguous_leaves =
			SimpleSmt::<2>::with_contiguous_leaves(digests_to_words(&VALUES_4))?;

		assert_eq!(tree_with_leaves, tree_with_contiguous_leaves);

		Ok(())
	}

	#[test]
	fn depth2_tree() -> Result<(), MerkleError> {
		let tree =
			SimpleSmt::<2>::with_leaves(KEYS_4.into_iter().zip(digests_to_words(&VALUES_4)))?;

		let (root, node2, node3) = compute_internal_nodes();
		assert_eq!(root, tree.root());
		assert_eq!(node2, tree.get_node(NodeIndex::new(1, 0)?)?);
		assert_eq!(node3, tree.get_node(NodeIndex::new(1, 1)?)?);

		assert_eq!(VALUES_4[0], tree.get_node(NodeIndex::new(2, 0)?)?);
		assert_eq!(VALUES_4[1], tree.get_node(NodeIndex::new(2, 1)?)?);
		assert_eq!(VALUES_4[2], tree.get_node(NodeIndex::new(2, 2)?)?);
		assert_eq!(VALUES_4[3], tree.get_node(NodeIndex::new(2, 3)?)?);

		assert_eq!(
			vec![VALUES_4[1], node3],
			*tree.open(&LeafIndex::new(0)?).path
		);
		assert_eq!(
			vec![VALUES_4[0], node3],
			*tree.open(&LeafIndex::new(1)?).path
		);
		assert_eq!(
			vec![VALUES_4[3], node2],
			*tree.open(&LeafIndex::new(2)?).path
		);
		assert_eq!(
			vec![VALUES_4[2], node2],
			*tree.open(&LeafIndex::new(3)?).path
		);

		Ok(())
	}

	#[test]
	fn inner_node_iterator() -> Result<(), MerkleError> {
		let tree =
			SimpleSmt::<2>::with_leaves(KEYS_4.into_iter().zip(digests_to_words(&VALUES_4)))?;

		assert_eq!(VALUES_4[0], tree.get_node(NodeIndex::new(2, 0)?)?);
		assert_eq!(VALUES_4[1], tree.get_node(NodeIndex::new(2, 1)?)?);
		assert_eq!(VALUES_4[2], tree.get_node(NodeIndex::new(2, 2)?)?);
		assert_eq!(VALUES_4[3], tree.get_node(NodeIndex::new(2, 3)?)?);

		let root = tree.root();
		let l1n0 = tree.get_node(NodeIndex::new(1, 0)?)?;
		let l1n1 = tree.get_node(NodeIndex::new(1, 1)?)?;
		let l2n0 = tree.get_node(NodeIndex::new(2, 0)?)?;
		let l2n1 = tree.get_node(NodeIndex::new(2, 1)?)?;
		let l2n2 = tree.get_node(NodeIndex::new(2, 2)?)?;
		let l2n3 = tree.get_node(NodeIndex::new(2, 3)?)?;

		let nodes: Vec<_> = tree.inner_nodes().collect();
		let expected = [
			InnerNodeInfo {
				value: root,
				left: l1n0,
				right: l1n1,
			},
			InnerNodeInfo {
				value: l1n0,
				left: l2n0,
				right: l2n1,
			},
			InnerNodeInfo {
				value: l1n1,
				left: l2n2,
				right: l2n3,
			},
		];

		assert_eq!(nodes, expected);

		Ok(())
	}

	#[test]
	fn insert() -> Result<(), MerkleError> {
		const DEPTH: u8 = 3;
		let mut tree =
			SimpleSmt::<DEPTH>::with_leaves(KEYS_8.into_iter().zip(digests_to_words(&VALUES_8)))?;
		assert_eq!(tree.num_leaves(), 8);

		let key = 3;
		let new_node = int_to_leaf(9);
		let mut expected_values = digests_to_words(&VALUES_8);
		expected_values[key] = new_node;
		let expected_tree = MerkleTree::new(expected_values.clone())?;

		let old_leaf = tree.insert(LeafIndex::new(key as u64)?, new_node);
		assert_eq!(tree.root(), expected_tree.root());
		assert_eq!(old_leaf, *VALUES_8[key]);
		assert_eq!(tree.num_leaves(), 8);

		let key = 6;
		let new_node = int_to_leaf(10);
		expected_values[key] = new_node;
		let expected_tree = MerkleTree::new(expected_values.clone())?;

		let old_leaf = tree.insert(LeafIndex::new(key as u64)?, new_node);
		assert_eq!(tree.root(), expected_tree.root());
		assert_eq!(old_leaf, *VALUES_8[key]);
		assert_eq!(tree.num_leaves(), 8);

		let key = 5;
		let new_node = EMPTY_WORD;
		expected_values[key] = new_node;
		let expected_tree = MerkleTree::new(expected_values.clone())?;

		let old_leaf = tree.insert(LeafIndex::new(key as u64)?, new_node);
		assert_eq!(tree.root(), expected_tree.root());
		assert_eq!(old_leaf, *VALUES_8[key]);
		assert_eq!(tree.num_leaves(), 7);

		Ok(())
	}

	#[test]
	fn small_tree_opening_is_consistent() -> Result<(), MerkleError> {
		let z = EMPTY_WORD;

		let a = Word::from(Rpo256::merge(&[z.into(); 2]));
		let b = Word::from(Rpo256::merge(&[a.into(); 2]));
		let c = Word::from(Rpo256::merge(&[b.into(); 2]));
		let d = Word::from(Rpo256::merge(&[c.into(); 2]));

		let e = Rpo256::merge(&[a.into(), b.into()]);
		let f = Rpo256::merge(&[z.into(); 2]);
		let g = Rpo256::merge(&[c.into(), z.into()]);
		let h = Rpo256::merge(&[z.into(), d.into()]);

		let i = Rpo256::merge(&[e, f]);
		let j = Rpo256::merge(&[g, h]);

		let k = Rpo256::merge(&[i, j]);

		let entries = [(0, a), (1, b), (4, c), (7, d)];
		let tree = SimpleSmt::<3>::with_leaves(entries)?;

		assert_eq!(tree.root(), k);

		let cases = [
			(0, vec![b.into(), f, j]),
			(1, vec![a.into(), f, j]),
			(4, vec![z.into(), h, i]),
			(7, vec![z.into(), g, i]),
		];

		for (key, path) in cases {
			let opening = tree.open(&LeafIndex::new(key)?);

			assert_eq!(*opening.path, path);
		}

		Ok(())
	}

	#[test]
	fn simple_smt_fail_on_duplicates() {
		let values = [
			(int_to_leaf(1), int_to_leaf(1)),
			(int_to_leaf(1), int_to_leaf(2)),
			(EMPTY_WORD, int_to_leaf(1)),
			(int_to_leaf(1), EMPTY_WORD),
			(EMPTY_WORD, EMPTY_WORD),
		];

		for (first, second) in values.iter().copied() {
			let entries = [(1, first), (1, second)];
			let smt = SimpleSmt::<64>::with_leaves(entries);
			assert_matches!(smt, Err(MerkleError::DuplicateValuesForIndex(1)));

			let entries = [(1, first), (5, int_to_leaf(5)), (1, second)];
			let smt = SimpleSmt::<64>::with_leaves(entries);
			assert_matches!(smt, Err(MerkleError::DuplicateValuesForIndex(1)));
		}
	}

	#[test]
	fn with_no_duplicates_empty_node() {
		let entries = [(1, int_to_leaf(0)), (5, int_to_leaf(2))];
		let smt = SimpleSmt::<64>::with_leaves(entries);
		assert!(smt.is_ok());
	}

	#[test]
	fn simple_smt_with_leaves_nonexisting_leaf() {
		let leaves = [(2, EMPTY_WORD)];
		let result = SimpleSmt::<1>::with_leaves(leaves);
		assert!(result.is_err());

		let leaves = [(4, EMPTY_WORD)];
		let result = SimpleSmt::<2>::with_leaves(leaves);
		assert!(result.is_err());

		let leaves = [(8, EMPTY_WORD)];
		let result = SimpleSmt::<3>::with_leaves(leaves);
		assert!(result.is_err());

		let value = int_to_node(1);

		let leaves = [(2, *value)];
		let result = SimpleSmt::<1>::with_leaves(leaves);
		assert!(result.is_err());

		let leaves = [(4, *value)];
		let result = SimpleSmt::<2>::with_leaves(leaves);
		assert!(result.is_err());

		let leaves = [(8, *value)];
		let result = SimpleSmt::<3>::with_leaves(leaves);
		assert!(result.is_err());
	}

	#[test]
	fn simple_smt_set_subtree() -> Result<(), MerkleError> {
		const TREE_DEPTH: u8 = 3;

		let z = EMPTY_WORD;

		let a = Word::from(Rpo256::merge(&[z.into(); 2]));
		let b = Word::from(Rpo256::merge(&[a.into(); 2]));
		let c = Word::from(Rpo256::merge(&[b.into(); 2]));
		let d = Word::from(Rpo256::merge(&[c.into(); 2]));

		let e = Rpo256::merge(&[a.into(), b.into()]);
		let f = Rpo256::merge(&[z.into(); 2]);
		let g = Rpo256::merge(&[c.into(), z.into()]);
		let h = Rpo256::merge(&[z.into(), d.into()]);

		let i = Rpo256::merge(&[e, f]);
		let j = Rpo256::merge(&[g, h]);

		let k = Rpo256::merge(&[i, j]);

		let subtree = {
			let entries = [(0, c)];
			SimpleSmt::<1>::with_leaves(entries)?
		};

		let tree = {
			let entries = [(0, a), (1, b), (7, d)];
			let mut tree = SimpleSmt::<TREE_DEPTH>::with_leaves(entries)?;

			tree.set_subtree(2, subtree)?;

			tree
		};

		assert_eq!(tree.root(), k);
		assert_eq!(tree.get_leaf(&LeafIndex::new(4)?), c);
		assert_eq!(
			tree.get_inner_node(NodeIndex::new_unchecked(2, 2)).hash(),
			g
		);

		Ok(())
	}

	#[test]
	fn simple_smt_set_subtree_unchanged_for_wrong_index() -> Result<(), MerkleError> {
		let z = EMPTY_WORD;

		let a = Word::from(Rpo256::merge(&[z.into(); 2]));
		let b = Word::from(Rpo256::merge(&[a.into(); 2]));
		let c = Word::from(Rpo256::merge(&[b.into(); 2]));
		let d = Word::from(Rpo256::merge(&[c.into(); 2]));

		let subtree = {
			let entries = [(0, c)];
			SimpleSmt::<1>::with_leaves(entries)?
		};

		let mut tree = {
			let entries = [(0, a), (1, b), (7, d)];
			SimpleSmt::<3>::with_leaves(entries)?
		};

		let tree_root_before_insertion = tree.root();

		assert!(tree.set_subtree(500, subtree).is_err());

		assert_eq!(tree.root(), tree_root_before_insertion);

		Ok(())
	}

	#[test]
	fn simple_smt_set_subtree_entire_tree() -> Result<(), MerkleError> {
		const DEPTH: u8 = 3;
		let z = EMPTY_WORD;

		let a = Word::from(Rpo256::merge(&[z.into(); 2]));
		let b = Word::from(Rpo256::merge(&[a.into(); 2]));
		let c = Word::from(Rpo256::merge(&[b.into(); 2]));
		let d = Word::from(Rpo256::merge(&[c.into(); 2]));

		let subtree = SimpleSmt::<DEPTH>::with_leaves([])?;
		assert_eq!(subtree.root(), *EmptySubtreeRoots::entry(DEPTH, 0));

		let mut tree = {
			let entries = [(0, a), (1, b), (4, c), (7, d)];
			SimpleSmt::<DEPTH>::with_leaves(entries)?
		};

		tree.set_subtree(0, subtree)?;

		assert_eq!(tree.root(), *EmptySubtreeRoots::entry(DEPTH, 0));

		Ok(())
	}

	#[test]
	fn simple_smt_check_empty_root_constant() {
		let empty_root_64_depth = EmptySubtreeRoots::empty_hashes(64)[0];
		assert_eq!(empty_root_64_depth, SimpleSmt::<64>::EMPTY_ROOT);

		let empty_root_32_depth = EmptySubtreeRoots::empty_hashes(32)[0];
		assert_eq!(empty_root_32_depth, SimpleSmt::<32>::EMPTY_ROOT);

		let empty_root_1_depth = EmptySubtreeRoots::empty_hashes(1)[0];
		assert_eq!(empty_root_1_depth, SimpleSmt::<1>::EMPTY_ROOT);
	}
}
