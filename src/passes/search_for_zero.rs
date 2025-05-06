use std::borrow::Cow;

use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Clone, Copy)]
pub struct SearchForZeroPass;

impl LoopPass for SearchForZeroPass {
	fn run_pass(&self, loop_values: &[Instruction]) -> Option<Change> {
		if let [Instruction::Move(x)] = loop_values {
			Some(Change::ReplaceOne(Instruction::JumpToZero(*x)))
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("search for zero")
	}
}
