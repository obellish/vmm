use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use super::{InnerNodeInfo, MerkleError, NodeIndex};
use crate::{
	Word,
	hash::rpo::{Rpo256, RpoDigest},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct MerklePath {
	nodes: Vec<RpoDigest>,
}

impl MerklePath {
	#[must_use]
	pub fn new(nodes: Vec<RpoDigest>) -> Self {
		assert!(
			nodes.len() <= u8::MAX.into(),
			"merkle path may have at most 256 items"
		);
		Self { nodes }
	}

	#[must_use]
	pub fn depth(&self) -> u8 {
		self.nodes.len() as u8
	}

	#[must_use]
	pub fn nodes(&self) -> &[RpoDigest] {
		&self.nodes
	}

	pub fn compute_root(&self, index: u64, node: RpoDigest) -> Result<RpoDigest, MerkleError> {
		let mut index = NodeIndex::new(self.depth(), index)?;
		let root = self.nodes.iter().copied().fold(node, |node, sibling| {
			let input = index.build_node(node, sibling);
			index.move_up();
			Rpo256::merge(&input)
		});

		Ok(root)
	}

	pub fn verify(&self, index: u64, node: RpoDigest, root: RpoDigest) -> Result<(), MerkleError> {
		let computed_root = self.compute_root(index, node)?;
		if computed_root != root {
			return Err(MerkleError::ConflictingRoots {
				expected: root,
				actual: computed_root,
			});
		}

		Ok(())
	}

	pub fn inner_nodes(
		&self,
		index: u64,
		node: RpoDigest,
	) -> Result<InnerNodeIterator<'_>, MerkleError> {
		Ok(InnerNodeIterator {
			nodes: &self.nodes,
			index: NodeIndex::new(self.depth(), index)?,
			value: node,
		})
	}
}

impl Deref for MerklePath {
	type Target = Vec<RpoDigest>;

	fn deref(&self) -> &Self::Target {
		&self.nodes
	}
}

impl DerefMut for MerklePath {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.nodes
	}
}

impl Deserializable for MerklePath {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let count = source.read_u8()?.into();
		let nodes = source.read_many::<RpoDigest>(count)?;
		Ok(Self { nodes })
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MerklePath {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		serde::Deserialize::deserialize(deserializer).map(|nodes| Self { nodes })
	}
}

impl From<MerklePath> for Vec<RpoDigest> {
	fn from(value: MerklePath) -> Self {
		value.nodes
	}
}

impl FromIterator<RpoDigest> for MerklePath {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = RpoDigest>,
	{
		Self::new(iter.into_iter().collect())
	}
}

impl IntoIterator for MerklePath {
	type IntoIter = alloc::vec::IntoIter<RpoDigest>;
	type Item = RpoDigest;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl Serializable for MerklePath {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		assert!(
			self.nodes.len() <= u8::MAX.into(),
			"length enforced in the constructor"
		);
		target.write_u8(self.nodes.len() as u8);
		target.write_many(&self.nodes);
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for MerklePath {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serde::Serialize::serialize(&self.nodes, serializer)
	}
}

pub struct InnerNodeIterator<'a> {
	nodes: &'a [RpoDigest],
	index: NodeIndex,
	value: RpoDigest,
}

impl Iterator for InnerNodeIterator<'_> {
	type Item = InnerNodeInfo;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index.is_root() {
			None
		} else {
			let sibling_pos = self.nodes.len() - self.index.depth() as usize;
			let (left, right) = if self.index.is_value_odd() {
				(self.nodes[sibling_pos], self.value)
			} else {
				(self.value, self.nodes[sibling_pos])
			};

			self.value = Rpo256::merge(&[left, right]);
			self.index.move_up();

			Some(InnerNodeInfo {
				value: self.value,
				left,
				right,
			})
		}
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ValuePath {
	pub value: RpoDigest,
	pub path: MerklePath,
}

impl ValuePath {
	#[must_use]
	pub const fn new(value: RpoDigest, path: MerklePath) -> Self {
		Self { value, path }
	}
}

impl Deserializable for ValuePath {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let value = RpoDigest::read_from(source)?;
		let path = MerklePath::read_from(source)?;
		Ok(Self::new(value, path))
	}
}

impl From<(MerklePath, Word)> for ValuePath {
	fn from(value: (MerklePath, Word)) -> Self {
		Self::new(value.1.into(), value.0)
	}
}

impl Serializable for ValuePath {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.value.write_into(target);
		self.path.write_into(target);
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RootPath {
	pub root: RpoDigest,
	pub path: MerklePath,
}

impl Deserializable for RootPath {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let root = RpoDigest::read_from(source)?;
		let path = MerklePath::read_from(source)?;
		Ok(Self { root, path })
	}
}

impl Serializable for RootPath {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.root.write_into(target);
		self.path.write_into(target);
	}
}

#[cfg(test)]
mod tests {
	use crate::merkle::{MerkleError, MerklePath, int_to_node};

	#[test]
	fn inner_nodes() -> Result<(), MerkleError> {
		let nodes = vec![
			int_to_node(1),
			int_to_node(2),
			int_to_node(3),
			int_to_node(4),
		];
		let merkle_path = MerklePath::new(nodes);

		let index = 6;
		let node = int_to_node(5);
		let root = merkle_path.compute_root(index, node)?;

		let inner_root = merkle_path.inner_nodes(index, node)?.last().unwrap().value;

		assert_eq!(root, inner_root);

		Ok(())
	}
}
