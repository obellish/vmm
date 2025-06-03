use vmm_ir::{Instruction, Offset};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeClearUntilZeroPass;

impl LoopPass for OptimizeClearUntilZeroPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::SetVal {
					value: None,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(x)),
			] => Some(Change::replace(Instruction::clear_until_zero(*x))),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::SetVal {
					value: None,
					offset: None
				},
				Instruction::MovePtr(Offset::Relative(..))
			]
		)
	}
}
