mod ops;

use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	num::NonZeroU8,
};

use vmm_utils::GetOrZero as _;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
	value: Option<NonZeroU8>,
	index: Option<usize>,
}

impl Cell {
	#[inline]
	#[must_use]
	pub const fn new(value: u8) -> Self {
		Self::create(value, None)
	}

	#[inline]
	#[must_use]
	pub const fn with_index(value: u8, index: usize) -> Self {
		Self::create(value, Some(index))
	}

	pub const fn set_index(&mut self, index: usize) {
		self.index = Some(index);
	}

	#[must_use]
	pub const fn as_u8(&self) -> &u8 {
		unsafe { &*(&raw const self.value).cast::<u8>() }
	}

	pub fn as_mut_u8(&mut self) -> &mut u8 {
		unsafe { &mut *(&raw mut self.value).cast::<u8>() }
	}

	#[must_use]
	pub fn value(self) -> u8 {
		self.value.get_or_zero()
	}

	pub const fn set_value(&mut self, value: u8) {
		self.value = NonZeroU8::new(value);
	}

	pub const fn clear_value(&mut self) {
		self.set_value(0);
	}

	#[must_use]
	#[allow(clippy::trivially_copy_pass_by_ref)]
	pub fn is_zero(&self) -> bool {
		matches!(self.value(), 0)
	}

	#[inline]
	const fn create(value: u8, index: Option<usize>) -> Self {
		Self {
			value: NonZeroU8::new(value),
			index,
		}
	}
}

impl Debug for Cell {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Cell")
			.field("value", &self.value())
			.finish()
	}
}

impl Default for Cell {
	fn default() -> Self {
		Self::new(0)
	}
}

impl Display for Cell {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.value(), f)
	}
}

impl From<Cell> for Option<NonZeroU8> {
	fn from(value: Cell) -> Self {
		value.value
	}
}

impl From<Cell> for u8 {
	fn from(value: Cell) -> Self {
		value.value()
	}
}

impl From<NonZeroU8> for Cell {
	fn from(value: NonZeroU8) -> Self {
		Self::new(value.get())
	}
}

impl From<Option<NonZeroU8>> for Cell {
	fn from(value: Option<NonZeroU8>) -> Self {
		Self::new(value.get_or_zero())
	}
}

impl From<u8> for Cell {
	fn from(value: u8) -> Self {
		Self::new(value)
	}
}

impl PartialEq<u8> for Cell {
	fn eq(&self, other: &u8) -> bool {
		PartialEq::eq(&self.value(), other)
	}
}

impl PartialOrd<u8> for Cell {
	fn partial_cmp(&self, other: &u8) -> Option<core::cmp::Ordering> {
		Some(Ord::cmp(&self.value(), other))
	}
}

#[cfg(test)]
mod tests {
	use super::Cell;

	#[test]
	fn as_u8_works() {
		let mut value = Cell::new(8);

		assert_eq!(*value.as_u8(), 8);

		*value.as_mut_u8() = 0;

		assert_eq!(*value.as_u8(), 0);
	}
}
