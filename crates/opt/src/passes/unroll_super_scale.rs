use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollScaleAndPass;

impl PeepholePass for UnrollScaleAndPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action,
					factor: 1,
					offset,
				}),
			] => Some(Change::replace(match action {
				ScaleAnd::Fetch => Instruction::fetch_val(*offset),
				ScaleAnd::Move => Instruction::move_val(*offset),
				ScaleAnd::Take => Instruction::take_val(*offset),
			})),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::Super(SuperInstruction::ScaleAnd {
				factor: 1,
				..
			})]
		)
	}
}
