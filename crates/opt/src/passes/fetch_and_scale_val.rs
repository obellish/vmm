use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeFetchAndScaleValPass;

impl PeepholePass for OptimizeFetchAndScaleValPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(x),
				Instruction::Super(SuperInstruction::ScaleAnd {
					offset: y,
					factor,
					action: ScaleAnd::Move,
				}),
				Instruction::MovePtr(Offset::Relative(z)),
			] if *y == *z && -x == *y => Some(Change::replace(Instruction::fetch_and_scale_val(
				*factor, x,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(x),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: y,
					..
				}),
				Instruction::MovePtr(Offset::Relative(z))
			]
			if *y == *z && -x == *y
		)
	}
}
