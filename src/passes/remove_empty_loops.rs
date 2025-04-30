use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct RemoveEmptyLoopsPass;

impl PeepholePass for RemoveEmptyLoopsPass {
	type State = ();

	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction], (): ()) -> Option<Change> {
		if window == [Instruction::JumpRight, Instruction::JumpLeft] {
			Some(Change::Remove)
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("remove empty loops")
	}
}
