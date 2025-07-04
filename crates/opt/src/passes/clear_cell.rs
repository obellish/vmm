use vmm_ir::{Instruction, Offset};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeClearCellPass;

impl LoopPass for OptimizeClearCellPass {
	#[inline]
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::replace(Instruction::clear_val()))
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(1, Some(1))
	}

	#[inline]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[Instruction::IncVal {
				value: 1 | -1,
				offset: Offset(0)
			}]
		)
	}
}
