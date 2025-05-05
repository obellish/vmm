use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct CombineAddInstrPass;

impl PeepholePass for CombineAddInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		if let (Instruction::Add(i1), Instruction::Add(i2)) = (window[0], window[1]) {
			if i1 == -i2 {
				Some(Change::Remove)
			} else {
				Some(Change::ReplaceOne(Instruction::Add(i1 + i2)))
			}
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("combine add instructions")
	}
}
