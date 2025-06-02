use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

use super::{Instruction, IsZeroingCell, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockInstruction {
	/// A dynamic loop, which checks the current cell for zero before executing
	DynamicLoop(Vec<Instruction>),
	/// An if-non-zero block, which zeros out the cell after executing
	IfNz(Vec<Instruction>),
}

impl BlockInstruction {
	pub fn dynamic(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::DynamicLoop(i.into_iter().collect())
	}

	pub fn if_nz(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::IfNz(i.into_iter().collect())
	}
}

impl Display for BlockInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::DynamicLoop(instrs) => {
				f.write_str("dylop\n")?;
				for i in instrs {
					Display::fmt(&i, f)?;
					f.write_char('\n')?;
				}
				f.write_str("end dylop")?;
			}
			Self::IfNz(instrs) => {
				f.write_str("ifnz\n")?;
				for i in instrs {
					Display::fmt(&i, f)?;
					f.write_char('\n')?;
				}
				f.write_str("end ifnz")?;
			}
		}

		Ok(())
	}
}

impl IsZeroingCell for BlockInstruction {
	#[inline]
	fn is_zeroing_cell(&self) -> bool {
		true
	}
}

impl PtrMovement for BlockInstruction {
	#[inline]
	fn ptr_movement(&self) -> Option<isize> {
		match self {
			Self::DynamicLoop(instrs) | Self::IfNz(instrs) => instrs.ptr_movement(),
		}
	}
}
