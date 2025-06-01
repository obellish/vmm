use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

use super::{Instruction, IsZeroingCell, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LoopInstruction {
	Dynamic(Vec<Instruction>),
	IfNz(Vec<Instruction>),
}

impl LoopInstruction {
	pub fn dynamic(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::Dynamic(i.into_iter().collect())
	}

	pub fn if_nz(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::IfNz(i.into_iter().collect())
	}
}

impl Display for LoopInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Dynamic(instrs) => {
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

impl IsZeroingCell for LoopInstruction {
	fn is_zeroing_cell(&self) -> bool {
		true
	}
}

impl PtrMovement for LoopInstruction {
	fn ptr_movement(&self) -> Option<isize> {
		match self {
			Self::Dynamic(instrs) | Self::IfNz(instrs) => instrs.ptr_movement(),
		}
	}
}
