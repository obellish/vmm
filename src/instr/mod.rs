mod parse;

use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::parse::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	Move(isize),
	Add(i8),
	Set(u8),
	JumpToZero(isize),
	Clear,
	Write,
	Read,
	JumpRight,
	JumpLeft,
}

#[expect(clippy::trivially_copy_pass_by_ref)]
impl Instruction {
	#[must_use]
	pub const fn needs_input(&self) -> bool {
		matches!(self, Self::Read)
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Add(i) => {
				if *i > 0 {
					for _ in 0..(*i) {
						f.write_char('+')?;
					}
				} else {
					for _ in 0..(-*i) {
						f.write_char('-')?;
					}
				}
			}
			Self::Set(i) => {
				for _ in 0..(*i) {
					f.write_char('+')?;
				}
			}
			Self::JumpRight => f.write_char('[')?,
			Self::JumpLeft => f.write_char(']')?,
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

impl From<ParsedInstruction> for Instruction {
	fn from(value: ParsedInstruction) -> Self {
		match value {
			ParsedInstruction::Input => Self::Read,
			ParsedInstruction::Output => Self::Write,
			ParsedInstruction::JumpLeft => Self::JumpLeft,
			ParsedInstruction::JumpRight => Self::JumpRight,
			ParsedInstruction::MoveLeft => Self::Move(-1),
			ParsedInstruction::MoveRight => Self::Move(1),
			ParsedInstruction::Decrement => Self::Add(-1),
			ParsedInstruction::Increment => Self::Add(1),
		}
	}
}
