mod scale;

use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	num::NonZeroU8,
};

use serde::{Deserialize, Serialize};
use tap::prelude::*;

pub use self::scale::*;
use super::{IsOffsetable, IsZeroingCell, MinimumOutputs, Offset, PtrMovement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SuperInstruction {
	ScaleAnd {
		action: ScaleAnd,
		offset: Offset,
		factor: u8,
	},
	FindAndSetZero {
		offset: Offset,
		value: NonZeroU8,
	},
	SetUntilZero {
		value: Option<NonZeroU8>,
		offset: Offset,
	},
	FindCellByZero {
		jump_by: Offset,
		offset: Offset,
	},
	ShiftVals(Offset),
}

impl SuperInstruction {
	#[must_use]
	pub fn scale_and(factor: u8, offset: impl Into<Offset>, action: ScaleAnd) -> Self {
		Self::ScaleAnd {
			action,
			offset: offset.into(),
			factor,
		}
	}

	#[must_use]
	pub fn scale_and_move_val(factor: u8, offset: impl Into<Offset>) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Move)
	}

	#[must_use]
	pub fn scale_and_take_val(factor: u8, offset: impl Into<Offset>) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Take)
	}

	#[must_use]
	pub fn fetch_and_scale_val(factor: u8, offset: impl Into<Offset>) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Fetch)
	}

	#[must_use]
	pub fn scale_and_set_val(factor: u8, offset: impl Into<Offset>, value: NonZeroU8) -> Self {
		Self::scale_and(factor, offset, ScaleAnd::Set(value))
	}

	#[must_use]
	pub fn find_and_set_zero(value: NonZeroU8, offset: impl Into<Offset>) -> Self {
		Self::FindAndSetZero {
			offset: offset.convert::<Offset>(),
			value,
		}
	}

	#[must_use]
	pub fn set_until_zero(value: u8, offset: impl Into<Offset>) -> Self {
		Self::SetUntilZero {
			value: NonZeroU8::new(value),
			offset: offset.convert(),
		}
	}

	#[must_use]
	pub fn find_cell_by_zero(jump_by: impl Into<Offset>, offset: impl Into<Offset>) -> Self {
		Self::FindCellByZero {
			jump_by: jump_by.convert(),
			offset: offset.convert(),
		}
	}

	#[must_use]
	pub fn shift_vals(offset: impl Into<Offset>) -> Self {
		Self::ShiftVals(offset.convert())
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
			i => Debug::fmt(&i, f)?,
		}

		Ok(())
	}
}

impl IsOffsetable for SuperInstruction {
	fn is_offsetable(&self) -> bool {
		false
	}

	fn offset(&self) -> Option<Offset> {
		None
	}

	unsafe fn offset_unchecked(&self) -> Offset {
		unsafe { core::hint::unreachable_unchecked() }
	}

	fn set_offset(&mut self, _: Offset) {}
}

impl IsZeroingCell for SuperInstruction {
	#[inline]
	fn is_zeroing_cell(&self) -> bool {
		matches!(
			self,
			Self::ScaleAnd {
				action: ScaleAnd::Move,
				..
			} | Self::SetUntilZero { .. }
				| Self::ShiftVals(..)
		)
	}
}

impl MinimumOutputs for SuperInstruction {
	fn min_outputs(&self) -> usize {
		0
	}
}

impl PtrMovement for SuperInstruction {
	#[inline]
	fn ptr_movement(&self) -> Option<Offset> {
		match self {
			Self::ScaleAnd {
				action: ScaleAnd::Move | ScaleAnd::Fetch | ScaleAnd::Set(..),
				..
			} => Some(Offset(0)),
			Self::ScaleAnd {
				action: ScaleAnd::Take,
				offset,
				..
			} => Some(*offset),
			_ => None,
		}
	}
}
