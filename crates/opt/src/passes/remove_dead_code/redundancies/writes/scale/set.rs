use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantSetScaleValPass;

impl PeepholePass for RemoveRedundantSetScaleValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset,
					..
				}),
			] => Some(Change::swap([
				Instruction::clear_val(),
				Instruction::move_ptr(offset),
				Instruction::clear_val(),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					offset: Offset(0),
					value: None
				},
				Instruction::TakeVal(..)
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Take | ScaleAnd::Move,
						..
					})
			] | [
				Instruction::TakeVal(..)
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Take,
						..
					}),
				Instruction::SetVal {
					offset: Offset(0),
					..
				}
			]
		)
	}
}
