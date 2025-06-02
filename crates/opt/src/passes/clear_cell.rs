use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeClearCellPass;

impl LoopPass for OptimizeClearCellPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1 | 1,
					offset: None,
				},
			] => Some(Change::Replace(Instruction::clear_val())),
			_ => None,
		}
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
