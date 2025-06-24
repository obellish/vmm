use core::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

use super::Offset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Value<T> {
	CellAt(Offset),
	Constant(T),
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
