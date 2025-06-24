use alloc::vec::Vec;
use core::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use super::{IsZeroingCell, Offset, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WriteInstruction {
	Cell {
		count: usize,
		offset: Offset,
	},
	CellAndSet {
		count: usize,
		offset: Offset,
		value: Option<NonZeroU8>,
	},
	Byte(u8),
	Bytes(Vec<u8>),
}

impl WriteInstruction {
	#[must_use]
	pub fn write_once() -> Self {
		Self::write_once_at(0)
	}

	#[must_use]
	pub fn write_once_at(offset: impl Into<Offset>) -> Self {
		Self::write_many_at(1, offset)
	}

	#[must_use]
	pub fn write_many(count: usize) -> Self {
		Self::write_many_at(count, 0)
	}

	#[must_use]
	pub fn write_many_at(count: usize, offset: impl Into<Offset>) -> Self {
		Self::Cell {
			count,
			offset: offset.into(),
		}
	}

	#[must_use]
	pub fn write_once_and_set(value: u8) -> Self {
		Self::write_once_and_set_at(0, value)
	}

	#[must_use]
	pub fn write_once_and_set_at(offset: impl Into<Offset>, value: u8) -> Self {
		Self::write_many_and_set_at(1, offset, value)
	}

	#[must_use]
	pub fn write_many_and_set(count: usize, value: u8) -> Self {
		Self::write_many_and_set_at(count, 0, value)
	}

	#[must_use]
	pub fn write_many_and_set_at(count: usize, offset: impl Into<Offset>, value: u8) -> Self {
		Self::CellAndSet {
			count,
			offset: offset.into(),
			value: NonZeroU8::new(value),
		}
	}

	#[must_use]
	pub const fn write_byte(c: u8) -> Self {
		Self::Byte(c)
	}

	#[must_use]
	pub fn write_bytes(s: impl IntoIterator<Item = u8>) -> Self {
		Self::Bytes(s.into_iter().collect())
	}
}

impl IsZeroingCell for WriteInstruction {
	fn is_zeroing_cell(&self) -> bool {
		matches!(self, Self::CellAndSet { value: None, .. } | Self::Byte(0))
	}
}

impl PtrMovement for WriteInstruction {
	fn ptr_movement(&self) -> Option<isize> {
		Some(0)
	}

	fn might_move_ptr(&self) -> bool {
		false
	}
}
