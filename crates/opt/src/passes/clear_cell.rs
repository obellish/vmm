use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeClearCellPass;

impl LoopPass for OptimizeClearCellPass {
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::replace(Instruction::clear_val()))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[Instruction::IncVal {
				value: -1 | 1,
				offset: None
			}]
		)
	}
}
