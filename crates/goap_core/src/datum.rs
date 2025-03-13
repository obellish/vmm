use std::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	hash::Hash,
	ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Serialize, Deserialize, Reflect)]
pub enum Datum {
	Bool(bool),
	I64(i64),
	F64(f64),
	Enum(usize),
}

impl Datum {
	#[allow(clippy::trivially_copy_pass_by_ref)]
	#[must_use]
	pub fn distance(&self, other: &Self) -> u64 {
		match (self, other) {
			(Self::Bool(l), Self::Bool(r)) if l == r => 0,
			(Self::I64(l), Self::I64(r)) => (l - r).unsigned_abs(),
			(Self::F64(l), Self::F64(r)) => (l - r).abs() as _,
			(Self::Enum(l), Self::Enum(r)) if l == r => 0,
			(Self::Enum(_), Self::Enum(_)) | (Self::Bool(_), Self::Bool(_)) => 1,
			_ => panic!("cannot calculate distance between two different datum types"),
		}
	}
}

impl Add for Datum {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::I64(l), Self::I64(r)) => Self::I64(l + r),
			(Self::F64(l), Self::F64(r)) => Self::F64(l + r),
			_ => panic!("unsupported operation: {self} + {rhs}"),
		}
	}
}

impl AddAssign for Datum {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl Display for Datum {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Datum:")?;
		match self {
			Self::Bool(b) => {
				f.write_str("Bool(")?;
				Display::fmt(&b, f)?;
			}
			Self::I64(i) => {
				f.write_str("I64(")?;
				Display::fmt(&i, f)?;
			}
			Self::F64(fv) => {
				f.write_str("F64(")?;
				Display::fmt(&fv, f)?;
			}
			Self::Enum(e) => {
				f.write_str("Enum(")?;
				Display::fmt(&e, f)?;
			}
		}
		f.write_char(')')
	}
}

impl Eq for Datum {}

impl From<bool> for Datum {
	fn from(value: bool) -> Self {
		Self::Bool(value)
	}
}

impl From<i64> for Datum {
	fn from(value: i64) -> Self {
		Self::I64(value)
	}
}

impl From<f64> for Datum {
	fn from(value: f64) -> Self {
		Self::F64(value)
	}
}

impl From<usize> for Datum {
	fn from(value: usize) -> Self {
		Self::Enum(value)
	}
}

impl Hash for Datum {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		std::mem::discriminant(self).hash(state);
		match self {
			Self::Bool(b) => b.hash(state),
			Self::I64(i) => i.hash(state),
			Self::F64(f) => f.to_bits().hash(state),
			Self::Enum(e) => e.hash(state),
		}
	}
}

impl PartialEq for Datum {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Bool(l), Self::Bool(r)) => l == r,
			(Self::I64(l), Self::I64(r)) => l == r,
			(Self::F64(l), Self::F64(r)) => l == r,
			(Self::Enum(l), Self::Enum(r)) => l == r,
			_ => false,
		}
	}
}

impl Sub for Datum {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::I64(l), Self::I64(r)) => Self::I64(l - r),
			(Self::F64(l), Self::F64(r)) => Self::F64(l - r),
			_ => panic!("unsupported operation: {self} - {rhs}"),
		}
	}
}

impl SubAssign for Datum {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

#[cfg(test)]
mod tests {
	use super::Datum;

	#[test]
	fn equality() {
		assert_eq!(Datum::Bool(true), Datum::Bool(true));
		assert_eq!(Datum::I64(666), Datum::I64(666));
		assert_eq!(Datum::F64(666.66), Datum::F64(666.66));
	}

	#[test]
	fn distance() {
		assert_eq!(Datum::Bool(true).distance(&Datum::Bool(true)), 0);
		assert_eq!(Datum::Bool(false).distance(&Datum::Bool(false)), 0);
		assert_eq!(Datum::Bool(true).distance(&Datum::Bool(false)), 1);
		assert_eq!(Datum::Bool(false).distance(&Datum::Bool(true)), 1);

		assert_eq!(Datum::I64(0).distance(&Datum::I64(0)), 0);
		assert_eq!(Datum::I64(0).distance(&Datum::I64(10)), 10);
		assert_eq!(Datum::I64(5).distance(&Datum::I64(-5)), 10);
		assert_eq!(Datum::I64(10).distance(&Datum::I64(10)), 0);
		assert_eq!(Datum::I64(10).distance(&Datum::I64(0)), 10);
		assert_eq!(Datum::I64(-5).distance(&Datum::I64(5)), 10);

		assert_eq!(Datum::F64(0.0).distance(&Datum::F64(0.0)), 0);
		assert_eq!(Datum::F64(1.5).distance(&Datum::F64(1.5)), 0);
		assert_eq!(Datum::F64(0.0).distance(&Datum::F64(1.5)), 1);
		assert_eq!(Datum::F64(1.5).distance(&Datum::F64(0.0)), 1);
		assert_eq!(Datum::F64(-2.5).distance(&Datum::F64(2.5)), 5);
		assert_eq!(Datum::F64(2.5).distance(&Datum::F64(-2.5)), 5);
		assert_eq!(Datum::F64(2.88).distance(&Datum::F64(1.03)), 1);

		assert_eq!(Datum::Enum(0).distance(&Datum::Enum(0)), 0);
		assert_eq!(Datum::Enum(1).distance(&Datum::Enum(1)), 0);
		assert_eq!(Datum::Enum(0).distance(&Datum::Enum(1)), 1);
		assert_eq!(Datum::Enum(1).distance(&Datum::Enum(0)), 1);
		assert_eq!(Datum::Enum(1).distance(&Datum::Enum(5)), 1);
	}
}
