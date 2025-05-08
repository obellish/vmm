use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Clone, Copy)]
pub struct SetZeroPass;

impl LoopPass for SetZeroPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		// if let [Instruction::Inc(-1)] = loop_values {
		// 	Some(Change::ReplaceOne(Instruction::Set(0)))
		// } else {
		// 	None
		// }
		match loop_values {
			[Instruction::IncVal(-1 | 1)] => Some(Change::ReplaceOne(Instruction::SetVal(0))),
			_ => None,
		}
	}
}
