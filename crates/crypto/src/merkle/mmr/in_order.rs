use core::num::NonZeroUsize;

use crate::utils::{Deserializable, Serializable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct InOrderIndex {
	idx: usize,
}

impl InOrderIndex {
	#[must_use]
	pub const fn new(idx: NonZeroUsize) -> Self {
		Self { idx: idx.get() }
	}

	#[must_use]
	pub const fn from_leaf_pos(leaf: usize) -> Self {
		let pos = leaf + 1;
		Self { idx: pos * 2 - 1 }
	}

	#[must_use]
	pub const fn is_leaf(self) -> bool {
		matches!(self.idx & 1, 1)
	}

	#[must_use]
	pub fn is_left_child(self) -> bool {
		self.parent().left_child() == self
	}

	#[must_use]
	pub const fn level(self) -> u32 {
		self.idx.trailing_zeros()
	}

	#[must_use]
	pub const fn left_child(self) -> Self {
		let els = 1 << (self.level() - 1);
		Self {
			idx: self.idx - els,
		}
	}

	#[must_use]
	pub const fn right_child(self) -> Self {
		let els = 1 << (self.level() - 1);
		Self {
			idx: self.idx + els,
		}
	}

	#[must_use]
	pub const fn parent(self) -> Self {
		let target = self.level() + 1;
		let bit = 1 << target;
		let mask = bit - 1;
		let idx = self.idx ^ (self.idx & mask);
		Self { idx: idx | bit }
	}

	#[must_use]
	pub fn sibling(self) -> Self {
		let parent = self.parent();
		if self > parent {
			parent.left_child()
		} else {
			parent.right_child()
		}
	}

	#[must_use]
	pub const fn inner(self) -> u64 {
		self.idx as u64
	}
}

impl Deserializable for InOrderIndex {
	fn read_from<R: crate::utils::ByteReader>(
		source: &mut R,
	) -> Result<Self, crate::utils::DeserializationError> {
		let idx = source.read_usize()?;
		Ok(Self { idx })
	}
}

impl From<InOrderIndex> for u64 {
	fn from(value: InOrderIndex) -> Self {
		value.inner()
	}
}

impl Serializable for InOrderIndex {
	fn write_into<W: crate::utils::ByteWriter>(&self, target: &mut W) {
		target.write_usize(self.idx);
	}
}

#[cfg(test)]
mod tests {
	use proptest::prelude::*;

	use super::InOrderIndex;
	use crate::utils::{Deserializable, DeserializationError, Serializable};

	#[test]
	fn in_order_index_basic() {
		let left = InOrderIndex::from_leaf_pos(0);
		let right = InOrderIndex::from_leaf_pos(1);

		assert!(left.is_leaf());
		assert!(right.is_leaf());
		assert_eq!(left.parent(), right.parent());
		assert_eq!(left.parent().right_child(), right);
		assert_eq!(left, right.parent().left_child());
		assert_eq!(left.sibling(), right);
		assert_eq!(left, right.sibling());
	}

	#[test]
	fn in_order_index_serialization() -> Result<(), DeserializationError> {
		let index = InOrderIndex::from_leaf_pos(5);
		let bytes = index.to_bytes();
		let index2 = InOrderIndex::read_from_bytes(&bytes)?;
		assert_eq!(index, index2);

		Ok(())
	}

	proptest! {
		#[test]
		fn in_order_index_random(count in 1..1000usize) {
			let left_pos = count * 2;
			let right_pos = count * 2 + 1;

			let left = InOrderIndex::from_leaf_pos(left_pos);
			let right = InOrderIndex::from_leaf_pos(right_pos);

			assert!(left.is_leaf());
			assert!(right.is_leaf());
			assert_eq!(left.parent(), right.parent());
			assert_eq!(left.parent().right_child(), right);
			assert_eq!(left, right.parent().left_child());
			assert_eq!(left.sibling(), right);
			assert_eq!(left, right.sibling());
		}
	}
}
