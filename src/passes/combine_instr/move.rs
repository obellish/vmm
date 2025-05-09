use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default, Clone, Copy)]
pub struct CombineMovePtrInstrPass;

impl PeepholePass for CombineMovePtrInstrPass {
	const SIZE: usize = 2;

	#[tracing::instrument]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::MovePtr(i1), Instruction::MovePtr(i2)] = window {
			if *i1 == -*i2 {
				Some(Change::Remove)
			} else {
				Some(Change::ReplaceOne(Instruction::MovePtr(
					i1.wrapping_add(*i2),
				)))
			}
		} else {
			None
		}
	}
}
