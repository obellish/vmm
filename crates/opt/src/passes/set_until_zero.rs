use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeSetUntilZeroPass;

impl LoopPass for OptimizeSetUntilZeroPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::SetVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(x)),
			] => Some(Change::replace(Instruction::set_until_zero(
				value.get_or_zero(),
				*x,
			))),
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
				Instruction::SetVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(..))
			]
		)
	}
}
