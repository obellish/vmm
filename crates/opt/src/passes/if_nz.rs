use vmm_ir::{Instruction, Offset};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeIfNzPass;

impl LoopPass for OptimizeIfNzPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				rest @ ..,
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
			] => Some(Change::replace(Instruction::if_nz(rest.iter().cloned()))),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(1, None)
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				..,
				Instruction::SetVal {
					value: None,
					offset: Offset(0)
				}
			]
		)
	}
}
