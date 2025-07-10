#![allow(clippy::match_wildcard_for_single_variants)]

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Integer {
	Unsigned(u128),
	Signed(i128),
}

impl Debug for Integer {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for Integer {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Signed(i) => Display::fmt(&i, f),
			Self::Unsigned(i) => Display::fmt(&i, f),
		}
	}
}

impl From<u8> for Integer {
	fn from(value: u8) -> Self {
		Self::Unsigned(value.into())
	}
}

impl From<u16> for Integer {
	fn from(value: u16) -> Self {
		Self::Unsigned(value.into())
	}
}

impl From<u32> for Integer {
	fn from(value: u32) -> Self {
		Self::Unsigned(value.into())
	}
}

impl From<u64> for Integer {
	fn from(value: u64) -> Self {
		Self::Unsigned(value.into())
	}
}

impl From<u128> for Integer {
	fn from(value: u128) -> Self {
		Self::Unsigned(value)
	}
}

impl From<i8> for Integer {
	fn from(value: i8) -> Self {
		Self::Signed(value.into())
	}
}

impl From<i16> for Integer {
	fn from(value: i16) -> Self {
		Self::Signed(value.into())
	}
}

impl From<i32> for Integer {
	fn from(value: i32) -> Self {
		Self::Signed(value.into())
	}
}

impl From<i64> for Integer {
	fn from(value: i64) -> Self {
		Self::Signed(value.into())
	}
}

impl From<i128> for Integer {
	fn from(value: i128) -> Self {
		Self::Signed(value)
	}
}

impl PartialEq<u8> for Integer {
	fn eq(&self, other: &u8) -> bool {
		match self {
			Self::Unsigned(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<u16> for Integer {
	fn eq(&self, other: &u16) -> bool {
		match self {
			Self::Unsigned(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<u32> for Integer {
	fn eq(&self, other: &u32) -> bool {
		match self {
			Self::Unsigned(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<u64> for Integer {
	fn eq(&self, other: &u64) -> bool {
		match self {
			Self::Unsigned(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<u128> for Integer {
	fn eq(&self, other: &u128) -> bool {
		match self {
			Self::Unsigned(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<i8> for Integer {
	fn eq(&self, other: &i8) -> bool {
		match self {
			Self::Signed(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<i16> for Integer {
	fn eq(&self, other: &i16) -> bool {
		match self {
			Self::Signed(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<i32> for Integer {
	fn eq(&self, other: &i32) -> bool {
		match self {
			Self::Signed(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<i64> for Integer {
	fn eq(&self, other: &i64) -> bool {
		match self {
			Self::Signed(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}

impl PartialEq<i128> for Integer {
	fn eq(&self, other: &i128) -> bool {
		match self {
			Self::Signed(lhs) => PartialEq::eq(lhs, &(*other).into()),
			_ => false,
		}
	}
}
