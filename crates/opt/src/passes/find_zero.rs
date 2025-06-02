use vmm_ir::{Instruction, Offset};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeFindZeroPass;

impl LoopPass for OptimizeFindZeroPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[Instruction::MovePtr(Offset::Relative(x))] => {
				Some(Change::replace(Instruction::find_zero(*x)))
			}
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(loop_values, [Instruction::MovePtr(Offset::Relative(_))])
	}
}
