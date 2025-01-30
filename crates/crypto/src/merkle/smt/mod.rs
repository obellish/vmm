#[cfg(feature = "serde")]
mod serde;
mod simple;

use alloc::{collections::BTreeMap, vec::Vec};

pub use self::simple::SimpleSmt;
use super::{EmptySubtreeRoots, MerkleError, MerklePath, NodeIndex};
use crate::{
	hash::rpo::{Rpo256, RpoDigest},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

pub const SMT_MIN_DEPTH: u8 = 1;
pub const SMT_MAX_DEPTH: u8 = 64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InnerNode {
	pub left: RpoDigest,
	pub right: RpoDigest,
}

impl InnerNode {
	#[must_use]
	pub fn hash(self) -> RpoDigest {
		Rpo256::merge(&[self.left, self.right])
	}
}

impl Deserializable for InnerNode {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let left = source.read()?;
		let right = source.read()?;

		Ok(Self { left, right })
	}
}

impl Serializable for InnerNode {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.left.write_into(target);
		self.right.write_into(target);
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct LeafIndex<const DEPTH: u8> {
	index: NodeIndex,
}

impl<const DEPTH: u8> LeafIndex<DEPTH> {
	pub fn new(value: u64) -> Result<Self, MerkleError> {
		if DEPTH < SMT_MIN_DEPTH {
			return Err(MerkleError::DepthTooSmall(DEPTH));
		}

		Ok(Self {
			index: NodeIndex::new(DEPTH, value)?,
		})
	}

	#[must_use]
	pub const fn value(self) -> u64 {
		self.index.value()
	}
}

impl LeafIndex<SMT_MAX_DEPTH> {
	#[must_use]
	pub const fn new_max_depth(value: u64) -> Self {
		Self {
			index: NodeIndex::new_unchecked(SMT_MAX_DEPTH, value),
		}
	}
}

impl<const DEPTH: u8> Deserializable for LeafIndex<DEPTH> {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		Ok(Self {
			index: source.read()?,
		})
	}
}

impl<const DEPTH: u8> From<LeafIndex<DEPTH>> for NodeIndex {
	fn from(value: LeafIndex<DEPTH>) -> Self {
		value.index
	}
}

impl<const DEPTH: u8> Serializable for LeafIndex<DEPTH> {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.index.write_into(target);
	}
}

impl<const DEPTH: u8> TryFrom<NodeIndex> for LeafIndex<DEPTH> {
	type Error = MerkleError;

	fn try_from(value: NodeIndex) -> Result<Self, Self::Error> {
		if value.depth() != DEPTH {
			return Err(MerkleError::InvalidNodeIndexDepth {
				expected: DEPTH,
				provided: value.depth(),
			});
		}

		Self::new(value.value())
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MutationSet<K, V, const DEPTH: u8> {
	old_root: RpoDigest,
	node_mutations: BTreeMap<NodeIndex, NodeMutation>,
	new_pairs: BTreeMap<K, V>,
	new_root: RpoDigest,
}

impl<K, V, const DEPTH: u8> MutationSet<K, V, DEPTH> {
	#[must_use]
	pub const fn root(&self) -> RpoDigest {
		self.new_root
	}

	#[must_use]
	pub const fn old_root(&self) -> RpoDigest {
		self.old_root
	}

	#[must_use]
	pub const fn node_mutations(&self) -> &BTreeMap<NodeIndex, NodeMutation> {
		&self.node_mutations
	}

	#[must_use]
	pub const fn new_pairs(&self) -> &BTreeMap<K, V> {
		&self.new_pairs
	}
}

impl<K, V: Deserializable, const DEPTH: u8> Deserializable for MutationSet<K, V, DEPTH>
where
	K: Deserializable + Ord,
{
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let old_root = source.read()?;
		let new_root = source.read()?;
		let node_mutations = source.read()?;
		let new_pairs = source.read()?;

		Ok(Self {
			old_root,
			node_mutations,
			new_pairs,
			new_root,
		})
	}
}

impl<K: Serializable, V: Serializable, const DEPTH: u8> Serializable for MutationSet<K, V, DEPTH> {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write(self.old_root());
		target.write(self.root());
		self.node_mutations().write_into(target);
		self.new_pairs().write_into(target);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeMutation {
	Removal,
	Addition(InnerNode),
}

impl Deserializable for NodeMutation {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		Ok(if source.read_bool()? {
			let inner_node = source.read()?;
			Self::Addition(inner_node)
		} else {
			Self::Removal
		})
	}
}

impl Serializable for NodeMutation {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		match self {
			Self::Removal => target.write_bool(false),
			Self::Addition(inner_node) => {
				target.write_bool(true);
				inner_node.write_into(target);
			}
		}
	}
}

pub(crate) trait SparseMerkleTree<const DEPTH: u8> {
	type Key: Clone + Ord;
	type Value: Clone + PartialEq;
	type Leaf: Clone;
	type Opening;

	const EMPTY_VALUE: Self::Value;
	const EMPTY_ROOT: RpoDigest;

	fn root(&self) -> RpoDigest;

	fn set_root(&mut self, root: RpoDigest);

	fn get_inner_node(&self, index: NodeIndex) -> InnerNode;

	fn insert_inner_node(&mut self, index: NodeIndex, inner_node: InnerNode) -> Option<InnerNode>;

	fn remove_inner_node(&mut self, index: NodeIndex) -> Option<InnerNode>;

	fn insert_value(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;

	fn get_value(&self, key: &Self::Key) -> Self::Value;

	fn get_leaf(&self, key: &Self::Key) -> Self::Leaf;

	fn hash_leaf(leaf: &Self::Leaf) -> RpoDigest;

	fn construct_prospective_leaf(
		&self,
		existing_leaf: Self::Leaf,
		key: &Self::Key,
		value: &Self::Value,
	) -> Self::Leaf;

	fn key_to_leaf_index(key: &Self::Key) -> LeafIndex<DEPTH>;

	fn path_and_leaf_to_opening(path: MerklePath, leaf: Self::Leaf) -> Self::Opening;

	fn open(&self, key: &Self::Key) -> Self::Opening {
		let leaf = self.get_leaf(key);

		let mut index: NodeIndex = {
			let leaf_index: LeafIndex<DEPTH> = Self::key_to_leaf_index(key);
			leaf_index.into()
		};

		let merkle_path = {
			let mut path = Vec::with_capacity(index.depth() as usize);
			for _ in 0..index.depth() {
				let is_right = index.is_value_odd();
				index.move_up();
				let InnerNode { left, right } = self.get_inner_node(index);
				let value = if is_right { left } else { right };
				path.push(value);
			}

			MerklePath::new(path)
		};

		Self::path_and_leaf_to_opening(merkle_path, leaf)
	}

	fn insert(&mut self, key: Self::Key, value: Self::Value) -> Self::Value {
		let old_value = self
			.insert_value(key.clone(), value.clone())
			.unwrap_or(Self::EMPTY_VALUE);

		if value == old_value {
			return value;
		}

		let leaf = self.get_leaf(&key);
		let node_index = {
			let leaf_index = Self::key_to_leaf_index(&key);
			leaf_index.into()
		};

		self.recompute_nodes_from_index_to_root(node_index, Self::hash_leaf(&leaf));

		old_value
	}

	fn recompute_nodes_from_index_to_root(
		&mut self,
		mut index: NodeIndex,
		node_hash_at_index: RpoDigest,
	) {
		let mut node_hash = node_hash_at_index;
		for node_depth in (0..index.depth()).rev() {
			let is_right = index.is_value_odd();
			index.move_up();
			let InnerNode { left, right } = self.get_inner_node(index);
			let (left, right) = if is_right {
				(left, node_hash)
			} else {
				(node_hash, right)
			};
			node_hash = Rpo256::merge(&[left, right]);

			if node_hash == *EmptySubtreeRoots::entry(DEPTH, node_depth) {
				self.remove_inner_node(index);
			} else {
				self.insert_inner_node(index, InnerNode { left, right });
			}
		}

		self.set_root(node_hash);
	}

	#[allow(clippy::map_unwrap_or)]
	fn compute_mutations(
		&self,
		kv_pairs: impl IntoIterator<Item = (Self::Key, Self::Value)>,
	) -> MutationSet<Self::Key, Self::Value, DEPTH> {
		let mut new_root = self.root();
		let mut new_pairs = BTreeMap::new();
		let mut node_mutations = BTreeMap::new();

		for (key, value) in kv_pairs {
			let old_value = new_pairs
				.get(&key)
				.cloned()
				.unwrap_or_else(|| self.get_value(&key));
			if value == old_value {
				continue;
			}

			let leaf_index = Self::key_to_leaf_index(&key);
			let mut node_index = NodeIndex::from(leaf_index);

			let old_leaf = {
				let pairs_at_index = new_pairs
					.iter()
					.filter(|&(new_key, _)| Self::key_to_leaf_index(new_key) == leaf_index);

				pairs_at_index.fold(self.get_leaf(&key), |acc, (k, v)| {
					let existing_leaf = acc;
					self.construct_prospective_leaf(existing_leaf, k, v)
				})
			};

			let new_leaf = self.construct_prospective_leaf(old_leaf, &key, &value);

			let mut new_child_hash = Self::hash_leaf(&new_leaf);

			for node_depth in (0..node_index.depth()).rev() {
				let is_right = node_index.is_value_odd();
				node_index.move_up();

				let old_node = node_mutations
					.get(&node_index)
					.map(|mutation| match mutation {
						NodeMutation::Addition(node) => *node,
						NodeMutation::Removal => {
							EmptySubtreeRoots::get_inner_node(DEPTH, node_depth)
						}
					})
					.unwrap_or_else(|| self.get_inner_node(node_index));

				let new_node = if is_right {
					InnerNode {
						left: old_node.left,
						right: new_child_hash,
					}
				} else {
					InnerNode {
						left: new_child_hash,
						right: old_node.right,
					}
				};

				new_child_hash = new_node.hash();

				let &equivalent_empty_hash = EmptySubtreeRoots::entry(DEPTH, node_depth);
				let is_removal = new_child_hash == equivalent_empty_hash;
				let new_entry = if is_removal {
					NodeMutation::Removal
				} else {
					NodeMutation::Addition(new_node)
				};
				node_mutations.insert(node_index, new_entry);
			}

			new_root = new_child_hash;

			new_pairs.insert(key, value);
		}

		MutationSet {
			old_root: self.root(),
			new_root,
			node_mutations,
			new_pairs,
		}
	}

	fn apply_mutations(
		&mut self,
		mutations: MutationSet<Self::Key, Self::Value, DEPTH>,
	) -> Result<(), MerkleError>
	where
		Self: Sized,
	{
		let MutationSet {
			old_root,
			new_pairs,
			node_mutations,
			new_root,
		} = mutations;

		if old_root != self.root() {
			return Err(MerkleError::ConflictingRoots {
				expected: self.root(),
				actual: old_root,
			});
		}

		for (index, mutation) in node_mutations {
			match mutation {
				NodeMutation::Removal => {
					self.remove_inner_node(index);
				}
				NodeMutation::Addition(node) => {
					self.insert_inner_node(index, node);
				}
			}
		}

		for (key, value) in new_pairs {
			self.insert_value(key, value);
		}

		self.set_root(new_root);

		Ok(())
	}

	fn apply_mutations_with_reversion(
		&mut self,
		mutations: MutationSet<Self::Key, Self::Value, DEPTH>,
	) -> Result<MutationSet<Self::Key, Self::Value, DEPTH>, MerkleError>
	where
		Self: Sized,
	{
		let MutationSet {
			old_root,
			node_mutations,
			new_pairs,
			new_root,
		} = mutations;

		if old_root != self.root() {
			return Err(MerkleError::ConflictingRoots {
				expected: self.root(),
				actual: old_root,
			});
		}

		let mut reverse_mutations = BTreeMap::new();
		for (index, mutation) in node_mutations {
			match mutation {
				NodeMutation::Removal => {
					if let Some(node) = self.remove_inner_node(index) {
						reverse_mutations.insert(index, NodeMutation::Addition(node));
					}
				}
				NodeMutation::Addition(node) => {
					if let Some(old_node) = self.insert_inner_node(index, node) {
						reverse_mutations.insert(index, NodeMutation::Addition(old_node));
					} else {
						reverse_mutations.insert(index, NodeMutation::Removal);
					}
				}
			}
		}

		let mut reverse_pairs = BTreeMap::new();
		for (key, value) in new_pairs {
			if let Some(old_value) = self.insert_value(key.clone(), value) {
				reverse_pairs.insert(key, old_value);
			} else {
				reverse_pairs.insert(key, Self::EMPTY_VALUE);
			}
		}

		self.set_root(new_root);

		Ok(MutationSet {
			old_root: new_root,
			node_mutations: reverse_mutations,
			new_pairs: reverse_pairs,
			new_root: old_root,
		})
	}
}
