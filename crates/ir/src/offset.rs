use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::*,
};

use serde::{Deserialize, Serialize};
use vmm_wrap::ops::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Offset {
	Relative(isize),
}

impl Offset {
	#[must_use]
	pub const fn abs(self) -> Self {
		match self {
			Self::Relative(i) => Self::Relative(i.abs()),
		}
	}

	#[must_use]
	pub const fn is_relative(self) -> bool {
		matches!(self, Self::Relative(_))
	}
}

impl Add for Offset {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Add::add(lhs, rhs))
		}
	}
}

impl Add<&Self> for Offset {
	type Output = Self;

	fn add(self, rhs: &Self) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<Offset> for &Offset {
	type Output = Offset;

	fn add(self, rhs: Offset) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add for &Offset {
	type Output = Offset;

	fn add(self, rhs: Self) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl Add<isize> for Offset {
	type Output = Self;

	fn add(self, rhs: isize) -> Self::Output {
		match self {
			Self::Relative(lhs) => Self::Relative(Add::add(lhs, rhs))
		}
	}
}

impl Add<&isize> for Offset {
	type Output = Self;

	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<isize> for &Offset {
	type Output = Offset;

	fn add(self, rhs: isize) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add<&isize> for &Offset {
	type Output = Offset;

	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl AddAssign for Offset {
	fn add_assign(&mut self, rhs: Self) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&Self> for Offset {
	fn add_assign(&mut self, rhs: &Self) {
		*self = Add::add(*self, *rhs);
	}
}

impl AddAssign<isize> for Offset {
	fn add_assign(&mut self, rhs: isize) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&isize> for Offset {
	fn add_assign(&mut self, rhs: &isize) {
		*self = Add::add(*self, *rhs);
	}
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let alt = f.alternate();

		match self {
			Self::Relative(offset) => {
				if alt {
					f.write_char('[')?;
				}

				Display::fmt(&offset, f)?;

				if alt {
					f.write_char(']')?;
				}
			}
		}

		Ok(())
	}
}

impl From<isize> for Offset {
	fn from(value: isize) -> Self {
		Self::Relative(value)
	}
}

impl From<&isize> for Offset {
	fn from(value: &isize) -> Self {
		(*value).into()
	}
}

impl Sub for Offset {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Sub::sub(lhs, rhs))
		}
	}
}

impl Sub<&Self> for Offset {
	type Output = Self;

	fn sub(self, rhs: &Self) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<Offset> for &Offset {
	type Output = Offset;

	fn sub(self, rhs: Offset) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub for &Offset {
	type Output = Offset;

	fn sub(self, rhs: Self) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}
