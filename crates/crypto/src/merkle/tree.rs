use alloc::{string::String, vec::Vec};
use core::{
	fmt::{Display, Error, Formatter, Result as FmtResult},
	ops::Deref,
	slice,
};

use super::{InnerNodeInfo, MerkleError, MerklePath, NodeIndex};
use crate::{
	Word,
	hash::rpo::{Rpo256, RpoDigest},
	utils::{uninit_vector, word_to_hex},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct MerkleTree {
	nodes: Vec<RpoDigest>,
}

impl MerkleTree {
	pub fn new<T>(leaves: T) -> Result<Self, MerkleError>
	where
		T: AsRef<[Word]>,
	{
		let leaves = leaves.as_ref();
		let n = leaves.len();
		if n <= 1 {
			return Err(MerkleError::DepthTooSmall(n as u8));
		} else if !n.is_power_of_two() {
			return Err(MerkleError::NumLeavesNotPowerOfTwo(n));
		}

		let mut nodes = unsafe { uninit_vector(2 * n) };
		nodes[0] = RpoDigest::default();

		nodes[n..].iter_mut().zip(leaves).for_each(|(node, leaf)| {
			*node = RpoDigest::from(*leaf);
		});

		let ptr = nodes.as_ptr().cast::<[RpoDigest; 2]>();
		let pairs = unsafe { slice::from_raw_parts(ptr, n) };

		for i in (1..n).rev() {
			nodes[i] = Rpo256::merge(&pairs[i]);
		}

		Ok(Self { nodes })
	}

	#[must_use]
	pub fn root(&self) -> RpoDigest {
		self.nodes[1]
	}

	#[must_use]
	pub fn depth(&self) -> u8 {
		(self.nodes.len() / 2).ilog2() as u8
	}

	pub fn get_node(&self, index: NodeIndex) -> Result<RpoDigest, MerkleError> {
		if index.is_root() {
			return Err(MerkleError::DepthTooSmall(index.depth()));
		} else if index.depth() > self.depth() {
			return Err(MerkleError::DepthTooBig(index.depth().into()));
		}

		let pos = index.to_scalar_index() as usize;
		Ok(self.nodes[pos])
	}

	pub fn get_path(&self, mut index: NodeIndex) -> Result<MerklePath, MerkleError> {
		if index.is_root() {
			return Err(MerkleError::DepthTooSmall(index.depth()));
		} else if index.depth() > self.depth() {
			return Err(MerkleError::DepthTooBig(index.depth().into()));
		}

		let mut path = Vec::with_capacity(index.depth() as usize);
		for _ in 0..index.depth() {
			let sibling = index.sibling().to_scalar_index() as usize;
			path.push(self.nodes[sibling]);
			index.move_up();
		}

		debug_assert!(
			index.is_root(),
			"the path walk must go all the way to the root"
		);

		Ok(path.into_iter().collect())
	}

	pub fn leaves(&self) -> impl Iterator<Item = (u64, &Word)> {
		let leaves_start = self.nodes.len() / 2;
		self.nodes
			.iter()
			.skip(leaves_start)
			.enumerate()
			.map(|(i, v)| (i as u64, &**v))
	}

	#[must_use]
	pub fn inner_nodes(&self) -> InnerNodeIterator<'_> {
		InnerNodeIterator {
			nodes: &self.nodes,
			index: 1,
		}
	}

	pub fn update_leaf<'a>(&'a mut self, index_value: u64, value: Word) -> Result<(), MerkleError> {
		let mut index = NodeIndex::new(self.depth(), index_value)?;

		debug_assert_eq!(self.nodes.len() & 1, 0);
		let n = self.nodes.len() / 2;

		let ptr = self.nodes.as_ptr().cast::<[RpoDigest; 2]>();
		let pairs: &'a [[RpoDigest; 2]] = unsafe { slice::from_raw_parts(ptr, n) };

		let pos = index.to_scalar_index() as usize;
		self.nodes[pos] = value.into();

		for _ in 0..index.depth() {
			index.move_up();
			let pos = index.to_scalar_index() as usize;
			let value = Rpo256::merge(&pairs[pos]);
			self.nodes[pos] = value;
		}

		Ok(())
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MerkleTree {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		serde::Deserialize::deserialize(deserializer).map(|nodes| Self { nodes })
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for MerkleTree {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serde::Serialize::serialize(&self.nodes, serializer)
	}
}

impl TryFrom<&[Word]> for MerkleTree {
	type Error = MerkleError;

	fn try_from(value: &[Word]) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl TryFrom<&[RpoDigest]> for MerkleTree {
	type Error = MerkleError;

	fn try_from(value: &[RpoDigest]) -> Result<Self, Self::Error> {
		let value = value.iter().map(|v| **v).collect::<Vec<_>>();
		Self::new(value)
	}
}

pub struct InnerNodeIterator<'a> {
	nodes: &'a [RpoDigest],
	index: usize,
}

impl Iterator for InnerNodeIterator<'_> {
	type Item = InnerNodeInfo;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.nodes.len() / 2 {
			let value = self.index;
			let left = self.index * 2;
			let right = left + 1;

			self.index += 1;

			Some(InnerNodeInfo {
				value: self.nodes[value],
				left: self.nodes[left],
				right: self.nodes[right],
			})
		} else {
			None
		}
	}
}

pub fn tree_to_text(tree: &MerkleTree) -> Result<String, Error> {
	let indent = "  ";
	let mut s = String::new();
	s.push_str(&word_to_hex(*tree.root())?);
	s.push('\n');
	for d in 1..=tree.depth() {
		let entries = 2u64.pow(d.into());
		for i in 0..entries {
			let index = NodeIndex::new(d, i).map_err(|_| Error)?;
			let node = tree.get_node(index).map_err(|_| Error)?;

			for _ in 0..d {
				s.push_str(indent);
			}
			s.push_str(&word_to_hex(*node)?);
			s.push('\n');
		}
	}

	Ok(s)
}

pub fn path_to_text(path: &MerklePath) -> Result<String, Error> {
	let mut s = String::new();
	s.push('[');

	for el in path.iter() {
		s.push_str(&word_to_hex(**el)?);
		s.push_str(", ");
	}

	if !matches!(path.len(), 0) {
		s.pop();
		s.pop();
	}
	s.push(']');

	Ok(s)
}

#[cfg(test)]
mod tests {
	use core::mem::size_of;

	use proptest::prelude::*;

	use super::*;
	use crate::{
		Felt, WORD_SIZE,
		merkle::{digests_to_words, int_to_leaf, int_to_node},
	};

	const LEAVES_4: [RpoDigest; WORD_SIZE] = [
		int_to_node(1),
		int_to_node(2),
		int_to_node(3),
		int_to_node(4),
	];

	const LEAVES_8: [RpoDigest; WORD_SIZE * 2] = [
		int_to_node(1),
		int_to_node(2),
		int_to_node(3),
		int_to_node(4),
		int_to_node(5),
		int_to_node(6),
		int_to_node(7),
		int_to_node(8),
	];

	fn compute_internal_nodes() -> (RpoDigest, RpoDigest, RpoDigest) {
		let node2 =
			Rpo256::hash_elements(&[Word::from(LEAVES_4[0]), Word::from(LEAVES_4[1])].concat());
		let node3 =
			Rpo256::hash_elements(&[Word::from(LEAVES_4[2]), Word::from(LEAVES_4[3])].concat());
		let root = Rpo256::merge(&[node2, node3]);

		(root, node2, node3)
	}

	#[test]
	fn build_merkle_tree() -> Result<(), MerkleError> {
		let tree = MerkleTree::new(digests_to_words(&LEAVES_4))?;
		assert_eq!(tree.nodes.len(), 8);

		for (a, b) in tree.nodes.iter().skip(4).zip(LEAVES_4.iter()) {
			assert_eq!(a, b);
		}

		let (root, node2, node3) = compute_internal_nodes();

		assert_eq!(root, tree.nodes[1]);
		assert_eq!(node2, tree.nodes[2]);
		assert_eq!(node3, tree.nodes[3]);

		assert_eq!(tree.root(), root);

		Ok(())
	}

	#[test]
	fn get_leaf() -> Result<(), MerkleError> {
		let tree = MerkleTree::new(digests_to_words(&LEAVES_4))?;

		let (_, node2, node3) = compute_internal_nodes();

		assert_eq!(
			vec![LEAVES_4[1], node3],
			*tree.get_path(NodeIndex::new(2, 0)?)?
		);
		assert_eq!(
			vec![LEAVES_4[0], node3],
			*tree.get_path(NodeIndex::new(2, 1)?)?
		);
		assert_eq!(
			vec![LEAVES_4[3], node2],
			*tree.get_path(NodeIndex::new(2, 2)?)?
		);
		assert_eq!(
			vec![LEAVES_4[2], node2],
			*tree.get_path(NodeIndex::new(2, 3)?)?
		);

		assert_eq!(vec![node3], *tree.get_path(NodeIndex::new(1, 0)?)?);
		assert_eq!(vec![node2], *tree.get_path(NodeIndex::new(1, 1)?)?);

		Ok(())
	}

	#[test]
	fn update_leaf() -> Result<(), MerkleError> {
		let mut tree = MerkleTree::new(digests_to_words(&LEAVES_8))?;

		let value = 3;
		let new_node = int_to_leaf(9);
		let mut expected_leaves = digests_to_words(&LEAVES_8);
		expected_leaves[value as usize] = new_node;
		let expected_tree = MerkleTree::new(expected_leaves.clone())?;

		tree.update_leaf(value, new_node)?;
		assert_eq!(tree.nodes, expected_tree.nodes);

		let value = 6;
		let new_node = int_to_leaf(10);
		expected_leaves[value as usize] = new_node;
		let expected_tree = MerkleTree::new(expected_leaves.clone())?;

		tree.update_leaf(value, new_node)?;
		assert_eq!(tree.nodes, expected_tree.nodes);

		Ok(())
	}

	#[test]
	fn nodes() -> Result<(), MerkleError> {
		let tree = MerkleTree::new(digests_to_words(&LEAVES_4))?;
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

	proptest! {
		#[test]
		fn arbitrary_word_can_be_represented_as_digest(
			a in prop::num::u64::ANY,
			b in prop::num::u64::ANY,
			c in prop::num::u64::ANY,
			d in prop::num::u64::ANY,
		) {
			let word = [Felt::new(a), Felt::new(b), Felt::new(c), Felt::new(d)];
			let digest = RpoDigest::from(word);

			let word_ptr = word.as_ptr().cast::<u8>();
			let digest_ptr = digest.as_ptr().cast::<u8>();
			assert_ne!(word_ptr, digest_ptr);

			let word_bytes = unsafe { slice::from_raw_parts(word_ptr ,size_of::<Word>()) };
			let digest_bytes = unsafe { slice::from_raw_parts(digest_ptr, size_of::<RpoDigest>()) };
			assert_eq!(word_bytes, digest_bytes);
		}
	}
}
