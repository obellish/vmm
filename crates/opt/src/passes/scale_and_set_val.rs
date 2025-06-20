use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeScaleAndSetValPass;

impl PeepholePass for OptimizeScaleAndSetValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset,
					factor,
				}),
				Instruction::SetVal {
					value: Some(value),
					offset: Offset(0),
				},
			] => Some(Change::replace(Instruction::scale_and_set_val(
				*factor, offset, *value,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					..
				}),
				Instruction::SetVal {
					offset: Offset(0),
					value: Some(..)
				}
			]
		)
	}
}
