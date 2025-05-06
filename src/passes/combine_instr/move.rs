use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct CombineMoveInstrPass;

impl PeepholePass for CombineMoveInstrPass {
	const SIZE: usize = 2;

	#[tracing::instrument]
	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::MovePtr(i1), Instruction::MovePtr(i2)] = window {
			if *i1 == -*i2 {
				Some(Change::Remove)
			} else {
				Some(Change::ReplaceOne(Instruction::MovePtr(
					i1.saturating_add(*i2),
				)))
			}
		} else {
			None
		}
	}
}
