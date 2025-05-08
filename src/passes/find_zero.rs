use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default, Clone, Copy)]
pub struct FindZeroPass;

impl LoopPass for FindZeroPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[Instruction::MovePtr(x)] => Some(Change::ReplaceOne(Instruction::FindZero(*x))),
			_ => None,
		}
	}
}
