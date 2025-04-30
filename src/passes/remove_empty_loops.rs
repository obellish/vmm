use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct RemoveEmptyLoopsPass;

impl PeepholePass for RemoveEmptyLoopsPass {
	type State = ();

	const SIZE: usize = 2;

<<<<<<< HEAD
	fn run_pass(&mut self, window: &[Instruction], (): ()) -> Option<Change> {
=======
	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
>>>>>>> parent of fea49c6 (more tracing and mutable passes)
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
