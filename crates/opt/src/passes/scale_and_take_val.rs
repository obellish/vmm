use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeScaleAndTakeValPass;

impl PeepholePass for OptimizeScaleAndTakeValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: x,
					factor,
				}),
				Instruction::MovePtr(y),
			] if *x == *y => Some(Change::replace(Instruction::scale_and_take_val(*factor, x))),
			[
				Instruction::TakeVal(offset),
				Instruction::ScaleVal { factor },
			] => Some(Change::replace(Instruction::scale_and_take_val(
				*factor, *offset,
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
					offset: x,
					..
				}),
				Instruction::MovePtr(y)
			]
			if *x == *y
		) || matches!(
			window,
			[Instruction::TakeVal(..), Instruction::ScaleVal { .. }]
		)
	}
}
