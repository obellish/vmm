use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct CombineZeroLoopInstrPass;

impl PeepholePass for CombineZeroLoopInstrPass {
	const SIZE: usize = 3;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		if matches!(
			(window[0], window[1], window[2]),
			(
				Instruction::JumpRight,
				Instruction::Add(_),
				Instruction::JumpLeft
			)
		) {
			Some(Change::ReplaceOne(Instruction::Set(0)))
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("combine zeroing-loop instructions")
	}
}
