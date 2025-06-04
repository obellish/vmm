use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};
use vmm_wrap::ops::{WrappingNeg, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Offset {
	Relative(isize),
	Absolute(usize),
}

impl Offset {
	#[must_use]
	pub const fn abs(self) -> Self {
		match self {
			Self::Relative(i) => Self::Relative(i.abs()),
			Self::Absolute(a) => Self::Absolute(a),
		}
	}

	#[must_use]
	pub const fn is_relative(self) -> bool {
		matches!(self, Self::Relative(_))
	}

	#[must_use]
	pub const fn is_absolute(self) -> bool {
		matches!(self, Self::Absolute(_))
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
			Self::Absolute(offset) => {
				if alt {
					f.write_char('{')?;
				}

				Display::fmt(&offset, f)?;

				if alt {
					f.write_char('}')?;
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

impl From<usize> for Offset {
	fn from(value: usize) -> Self {
		Self::Absolute(value)
	}
}

impl From<&usize> for Offset {
	fn from(value: &usize) -> Self {
		(*value).into()
	}
}

impl WrappingAdd for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(l), Self::Relative(r)) => Self::Relative(l.wrapping_add(r)),
			(Self::Absolute(l), Self::Absolute(r)) => Self::Absolute(l.wrapping_add(r)),
			(Self::Absolute(l), Self::Relative(r)) => {
				Self::Absolute(WrappingAdd::wrapping_add(l, r))
			}
			(Self::Relative(l), Self::Absolute(r)) => {
				Self::Relative(WrappingAdd::wrapping_add(l, r))
			}
		}
	}
}

impl WrappingAdd<Offset> for &Offset {
	type Output = <Offset as WrappingAdd>::Output;

	fn wrapping_add(self, rhs: Offset) -> Self::Output {
		(*self).wrapping_add(rhs)
	}
}

impl WrappingAdd<&Self> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: &Self) -> Self::Output {
		self.wrapping_add(*rhs)
	}
}

impl WrappingAdd for &Offset {
	type Output = <Offset as WrappingAdd>::Output;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		(*self).wrapping_add(*rhs)
	}
}

impl WrappingAdd<isize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: isize) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(WrappingAdd::wrapping_add(a, rhs)),
			Self::Relative(r) => Self::Relative(WrappingAdd::wrapping_add(r, rhs)),
		}
	}
}

impl WrappingNeg for Offset {
	type Output = Self;

	#[inline]
	fn wrapping_neg(self) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(a.wrapping_neg()),
			Self::Relative(r) => Self::Relative(r.wrapping_neg()),
		}
	}
}
