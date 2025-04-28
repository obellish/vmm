use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct RemoveEmptyLoopsPass;

impl PeepholePass for RemoveEmptyLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&self, window: &[Instruction]) -> Change {
		if window == [Instruction::JumpRight, Instruction::JumpLeft] {
			Change::Remove
		} else {
			Change::Ignore
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("remove empty loops")
	}
}
