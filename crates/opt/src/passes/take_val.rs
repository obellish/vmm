use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeTakeValPass;

impl PeepholePass for OptimizeTakeValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(x),
					factor,
				}),
				Instruction::MovePtr(Offset::Relative(y)),
			] if *x == *y => Some(Change::ReplaceOne(Instruction::scale_and_take_val(
				*factor, x,
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
					offset: Offset::Relative(x),
					..
				}),
				Instruction::MovePtr(Offset::Relative(y))
			]
			if *x == *y
		)
	}
}
