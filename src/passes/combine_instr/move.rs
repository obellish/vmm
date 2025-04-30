use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct CombineMoveInstrPass;

impl PeepholePass for CombineMoveInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let (Instruction::Move(i1), Instruction::Move(i2)) = (window[0], window[1]) {
			if i1 == -i2 {
				Some(Change::Remove)
			} else {
				Some(Change::ReplaceOne(Instruction::Move(i1 + i2)))
			}
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("combine move instructions")
	}
}
