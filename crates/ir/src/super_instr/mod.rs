mod scale;

use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::scale::*;
use super::{Instruction, Offset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SuperInstruction {
	ScaleAnd {
		action: ScaleAnd,
		offset: Offset,
		factor: u8,
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
}

impl Display for SuperInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::ScaleAnd {
				action: ScaleAnd::Move,
				offset: Offset::Relative(offset),
				factor,
			} => {
				f.write_str("[-")?;
				Display::fmt(&Instruction::MovePtr(Offset::Relative(offset)), f)?;

				for _ in 0..factor {
					f.write_char('+')?;
				}

				Display::fmt(&Instruction::MovePtr(Offset::Relative(-offset)), f)?;

				f.write_char(']')?;
			}
			Self::ScaleAnd {
				action: ScaleAnd::Fetch,
				offset: Offset::Relative(offset),
				factor,
			} => {
				f.write_char('[')?;

				Display::fmt(&Instruction::MovePtr(Offset::Relative(offset)), f)?;

				f.write_char('-')?;

				Display::fmt(&Instruction::MovePtr(Offset::Relative(-offset)), f)?;

				for _ in 0..factor {
					f.write_char('+')?;
				}

				f.write_char(']')?;
			}
			Self::ScaleAnd {
				action: ScaleAnd::Take,
				offset: Offset::Relative(offset),
				factor,
			} => {
				Display::fmt(
					&Self::ScaleAnd {
						action: ScaleAnd::Move,
						offset: Offset::Relative(offset),
						factor,
					},
					f,
				)?;

				Display::fmt(&Instruction::MovePtr(Offset::Relative(offset)), f)?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}
