use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	ops::Add,
};

use serde::{Deserialize, Serialize};
use vmm_num::ops::CheckedAdd;

use super::{Bytes, Offset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Value<T> {
	CellAt(Offset),
	Constant(T),
}

impl<T> Value<T> {
	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Value<U> {
		match self {
			Self::CellAt(offset) => Value::CellAt(offset),
			Self::Constant(v) => Value::Constant(f(v)),
		}
	}
}

impl<T> CheckedAdd for Value<T>
where
	T: Add<Output = T>,
{
	type Output = Self;

	fn checked_add(self, rhs: Self) -> Option<Self::Output> {
		match (self, rhs) {
			(Self::CellAt(left), Self::CellAt(right)) => Some(Self::CellAt(left + right)),
			(Self::Constant(left), Self::Constant(right)) => {
				Some(Self::Constant(Add::add(left, right)))
			}
			_ => None,
		}
	}
}

impl<T> Default for Value<T> {
	fn default() -> Self {
		Self::CellAt(Offset(0))
	}
}

impl<T: Display> Display for Value<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::CellAt(c) => Display::fmt(c, f),
			Self::Constant(v) => Display::fmt(v, f),
		}
	}
}

impl<T> From<T> for Value<T> {
	fn from(value: T) -> Self {
		Self::Constant(value)
	}
}

pub trait FromCell {
	fn from_cell(cell: u8) -> Self;
}

impl FromCell for u8 {
	fn from_cell(cell: u8) -> Self {
		cell
	}
}

impl FromCell for Bytes {
	fn from_cell(cell: u8) -> Self {
		Self::Single(cell)
	}
}

impl FromCell for i8 {
	fn from_cell(cell: u8) -> Self {
		cell as Self
	}
}

#[cfg(test)]
mod tests {
	use vmm_num::Checked;

	use super::Value;
	use crate::Offset;

	#[test]
	fn checked_add_works() {
		let left = Value::Constant(5u8);
		let right = Value::Constant(4u8);

		assert_eq!(Checked::add(left, right), Some(Value::Constant(9u8)));

		let left = Value::Constant(5u8);
		let right = Value::<u8>::CellAt(Offset(4));

		assert!(Checked::add(left, right).is_none());
	}
}
