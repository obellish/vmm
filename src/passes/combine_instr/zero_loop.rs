use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct CombineZeroLoopInstrPass;

impl PeepholePass for CombineZeroLoopInstrPass {
	type State = ();

	const SIZE: usize = 3;

<<<<<<< HEAD
	fn run_pass(&mut self, window: &[Instruction], (): ()) -> Option<Change> {
=======
	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
>>>>>>> parent of fea49c6 (more tracing and mutable passes)
		if matches!(
			(window[0], window[1], window[2]),
			(
				Instruction::JumpRight,
				Instruction::Add(_),
				Instruction::JumpLeft
			)
		) {
			Some(Change::ReplaceOne(Instruction::Clear))
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("combine zeroing-loop instructions")
	}
}
