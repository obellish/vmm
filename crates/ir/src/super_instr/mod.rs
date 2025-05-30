mod scale;

use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::scale::*;
use super::{Offset, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SuperInstruction {
	ScaleAnd {
		action: ScaleAnd,
		offset: Offset,
		factor: u8,
	},
	DuplicateVal {
		offsets: Vec<Offset>,
	},
}

impl SuperInstruction {
	pub fn scale_and(factor: u8, offset: impl Into<Offset>, action: ScaleAnd) -> Self {
		Self::ScaleAnd {
			action,
			offset: offset.into(),
			factor,
		}
	}

	pub fn scale_and_move_val(factor: u8, offset: impl Into<Offset>) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Move)
	}

	pub fn scale_and_take_val(factor: u8, offset: impl Into<Offset>) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Take)
	}

	pub fn fetch_and_scale_val(factor: u8, offset: impl Into<Offset>) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Fetch)
	}

	#[must_use]
	pub const fn dupe_val(offsets: Vec<Offset>) -> Self {
		Self::DuplicateVal { offsets }
	}
}

#[allow(unreachable_patterns)]
impl Display for SuperInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::ScaleAnd {
				action,
				offset,
				factor,
			} => {
				f.write_str("scaleand")?;
				Display::fmt(&action, f)?;
				f.write_char(' ')?;
				Display::fmt(&factor, f)?;
				if let Offset::Relative(offset) = offset {
					f.write_str(" [")?;
					Display::fmt(&offset, f)?;
					f.write_char(']')?;
				}
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

impl PtrMovement for SuperInstruction {
	fn ptr_movement(&self) -> Option<isize> {
		match self {
			Self::ScaleAnd {
				action: ScaleAnd::Move | ScaleAnd::Fetch,
				..
			} => Some(0),
			Self::ScaleAnd {
				action: ScaleAnd::Take,
				offset: Offset::Relative(offset),
				..
			} => Some(*offset),
			_ => None,
		}
	}
}
