mod parse;

use serde::{Deserialize, Serialize};

pub use self::parse::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	Move(isize),
	Add(i8),
	Set(u8),
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
			ParsedInstruction::Clear => Self::Clear
		}
	}
}
