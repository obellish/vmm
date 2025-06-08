mod scale;

use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	num::NonZeroU8,
};

use serde::{Deserialize, Serialize};

pub use self::scale::*;
use super::{IsZeroingCell, Offset, PtrMovement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SuperInstruction {
	ScaleAnd {
		action: ScaleAnd,
		offset: Offset,
		factor: u8,
	},
	FindAndSetZero {
		offset: isize,
		value: NonZeroU8,
	},
	SetUntilZero {
		value: Option<NonZeroU8>,
		offset: isize,
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
	pub const fn find_and_set_zero(value: NonZeroU8, offset: isize) -> Self {
		Self::FindAndSetZero { offset, value }
	}

	#[must_use]
	pub const fn set_until_zero(value: u8, offset: isize) -> Self {
		Self::SetUntilZero {
			value: NonZeroU8::new(value),
			offset,
		}
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
				let offset = offset.value();
				f.write_str(" [")?;
				Display::fmt(&offset, f)?;
				f.write_char(']')?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

impl IsZeroingCell for SuperInstruction {
	fn is_zeroing_cell(&self) -> bool {
		matches!(
			self,
			Self::ScaleAnd {
				action: ScaleAnd::Move,
				..
			} | Self::SetUntilZero { .. }
		)
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
				offset,
				..
			} => Some(offset.value()),
			_ => None,
		}
	}
}
