#[cfg(feature = "serde")]
mod serde;

use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};

use super::MerkleError;
use crate::{
	Felt,
	hash::rpo::RpoDigest,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex {
	depth: u8,
	value: u64,
}

impl NodeIndex {
	pub const fn new(depth: u8, value: u64) -> Result<Self, MerkleError> {
		if (64 - value.leading_zeros()) > depth as u32 {
			Err(MerkleError::InvalidNodeIndex { depth, value })
		} else {
			Ok(Self { depth, value })
		}
	}

	#[must_use]
	pub const fn new_unchecked(depth: u8, value: u64) -> Self {
		debug_assert!((64 - value.leading_zeros()) <= depth as u32);
		Self { depth, value }
	}

	pub fn from_elements(depth: Felt, value: Felt) -> Result<Self, MerkleError> {
		let depth = depth.as_int();
		let depth = u8::try_from(depth).map_err(|_| MerkleError::DepthTooBig(depth))?;
		let value = value.as_int();
		Self::new(depth, value)
	}

	#[must_use]
	pub const fn root() -> Self {
		Self { depth: 0, value: 0 }
	}

	#[must_use]
	pub const fn sibling(mut self) -> Self {
		self.value ^= 1;
		self
	}

	#[must_use]
	pub const fn left_child(mut self) -> Self {
		self.depth += 1;
		self.value <<= 1;
		self
	}

	#[must_use]
	pub const fn right_child(mut self) -> Self {
		self.depth += 1;
		self.value = (self.value << 1) + 1;
		self
	}

	#[must_use]
	pub const fn build_node(self, slf: RpoDigest, sibling: RpoDigest) -> [RpoDigest; 2] {
		if self.is_value_odd() {
			[sibling, slf]
		} else {
			[slf, sibling]
		}
	}

	#[must_use]
	pub const fn to_scalar_index(self) -> u64 {
		(1 << self.depth() as u64) + self.value()
	}

	#[must_use]
	pub const fn depth(self) -> u8 {
		self.depth
	}

	#[must_use]
	pub const fn value(self) -> u64 {
		self.value
	}

	#[must_use]
	pub const fn is_value_odd(self) -> bool {
		matches!(self.value() & 1, 1)
	}

	#[must_use]
	pub const fn is_root(self) -> bool {
		matches!(self.depth(), 0)
	}

	pub fn move_up(&mut self) {
		self.depth = self.depth.saturating_sub(1);
		self.value >>= 1;
	}

	pub fn move_up_to(&mut self, depth: u8) {
		debug_assert!(depth < self.depth);
		let delta = self.depth.saturating_sub(depth);
		self.depth = self.depth.saturating_sub(delta);
		self.value >>= u32::from(depth);
	}
}

impl Deserializable for NodeIndex {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let depth = source.read_u8()?;
		let value = source.read_u64()?;
		Self::new(depth, value)
			.map_err(|_| DeserializationError::InvalidValue(String::from("invalid index")))
	}
}

impl Display for NodeIndex {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("depth=")?;
		Display::fmt(&self.depth, f)?;
		f.write_str(", value=")?;
		Display::fmt(&self.value, f)
	}
}

impl Serializable for NodeIndex {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u8(self.depth);
		target.write_u64(self.value);
	}
}

#[cfg(test)]
mod tests {
	use assert_matches::assert_matches;
	use proptest::prelude::*;

	use super::*;

	#[test]
	fn node_index_value_too_high() -> Result<(), MerkleError> {
		assert_eq!(NodeIndex::new(0, 0)?, NodeIndex { depth: 0, value: 0 });
		let err = NodeIndex::new(0, 1).unwrap_err();
		assert_matches!(err, MerkleError::InvalidNodeIndex { depth: 0, value: 1 });

		assert_eq!(NodeIndex::new(1, 1)?, NodeIndex { depth: 1, value: 1 });
		let err = NodeIndex::new(1, 2).unwrap_err();
		assert_matches!(err, MerkleError::InvalidNodeIndex { depth: 1, value: 2 });

		assert_eq!(NodeIndex::new(2, 3)?, NodeIndex { depth: 2, value: 3 });
		let err = NodeIndex::new(2, 4).unwrap_err();
		assert_matches!(err, MerkleError::InvalidNodeIndex { depth: 2, value: 4 });

		assert_eq!(NodeIndex::new(3, 7)?, NodeIndex { depth: 3, value: 7 });
		let err = NodeIndex::new(3, 8).unwrap_err();
		assert_matches!(err, MerkleError::InvalidNodeIndex { depth: 3, value: 8 });

		Ok(())
	}

	#[test]
	fn node_index_can_represent_depth_64() {
		assert!(NodeIndex::new(64, u64::MAX).is_ok());
	}

	prop_compose! {
		fn node_index() (value in 0..2u64.pow(u64::BITS - 1)) -> NodeIndex {
			let mut depth = value.ilog2() as u8;
			if value > (1 << depth) {
				depth += 1;
			}

			NodeIndex::new(depth, value).unwrap()
		}
	}

	proptest! {
		#[test]
		fn arbitrary_index_wont_panic_on_move_up(mut index in node_index(), count in prop::num::u8::ANY) {
			for _ in 0..count {
				index.move_up();
			}
		}
	}
}
