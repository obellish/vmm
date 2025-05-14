use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct ClearCellPass;

impl LoopPass for ClearCellPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[Instruction::IncVal(-1 | 1)] => Some(Change::ReplaceOne(Instruction::SetVal(0))),
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(loop_values, [Instruction::IncVal(-1 | 1)])
	}
}
