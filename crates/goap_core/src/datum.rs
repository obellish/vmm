use std::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	hash::{Hash, Hasher},
	ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy_reflect::Reflect;

#[derive(Debug, Clone, Copy, PartialOrd, Reflect)]
pub enum Datum {
	Bool(bool),
	I64(i64),
	F64(f64),
	Enum(usize),
}

impl Datum {
	#[must_use]
	pub fn distance(&self, other: &Self) -> u64 {
		match (self, other) {
			(Self::Bool(a), Self::Bool(b)) if a == b => 0,
			(Self::I64(a), Self::I64(b)) => (a - b).unsigned_abs(),
			(Self::F64(a), Self::F64(b)) => (a - b).abs() as u64,
			(Self::Enum(a), Self::Enum(b)) if a == b => 0,
			(Self::Enum(_), Self::Enum(_)) | (Self::Bool(_), Self::Bool(_)) => 1,
			_ => panic!("cannot calculate distance between different Datum types"),
		}
	}
}

impl Add for Datum {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::I64(a), Self::I64(b)) => Self::I64(a + b),
			(Self::F64(a), Self::F64(b)) => Self::F64(a + b),
			_ => panic!("unsupported addition between Datum variants, {self:?} + {rhs:?}"),
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
			Self::F64(fl) => {
				f.write_str("F64(")?;
				Display::fmt(&fl, f)?;
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

impl Hash for Datum {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);

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
			(Self::I64(a), Self::I64(b)) => Self::I64(a - b),
			(Self::F64(a), Self::F64(b)) => Self::F64(a - b),
			_ => panic!("unsupported subtraction between Datum variants, {self:?} - {rhs:?}"),
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
		assert_eq!(Datum::F64(666.666), Datum::F64(666.666));
	}

	#[test]
	#[allow(clippy::neg_cmp_op_on_partial_ord)]
	fn ordering() {
		assert!(Datum::I64(100) > Datum::I64(10));
		assert!(Datum::I64(1) > Datum::I64(0));
		assert!(Datum::F64(1.2) > Datum::F64(1.1));

		assert!(Datum::I64(100) >= Datum::I64(10));
		assert!(Datum::I64(1) >= Datum::I64(0));
		assert!(!(Datum::I64(100) >= Datum::I64(101)));
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
