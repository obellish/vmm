use crate::{Change, Instruction, LoopPass, StackedInstruction};

#[derive(Debug, Default, Clone, Copy)]
pub struct SetZeroPass;

impl LoopPass for SetZeroPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[Instruction::Stacked(StackedInstruction::IncVal(-1 | 1))] => {
				Some(Change::ReplaceOne(Instruction::SetVal(0)))
			}
			_ => None,
		}
	}
}
