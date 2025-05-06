use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Clone, Copy)]
pub struct SearchForZeroPass;

impl LoopPass for SearchForZeroPass {
	fn run_pass(&self, loop_values: &[Instruction]) -> Option<Change> {
		if let [Instruction::MovePtr(x)] = loop_values {
			Some(Change::ReplaceOne(Instruction::FindZero(*x)))
		} else {
			None
		}
	}
}
