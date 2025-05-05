use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

// TODO: Think of a better name
#[derive(Debug)]
pub struct SearchForZeroPass;

impl PeepholePass for SearchForZeroPass {
	const SIZE: usize = 3;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		if let [
			Instruction::JumpRight,
			Instruction::Move(i),
			Instruction::JumpLeft,
		] = window
		{
			Some(Change::ReplaceOne(Instruction::JumpToZero(*i)))
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("optimize move until zero loops")
	}
}
