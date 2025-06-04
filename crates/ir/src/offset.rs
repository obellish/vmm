use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};
use vmm_wrap::ops::WrappingNeg;

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
