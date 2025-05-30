use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeFetchValPass;

impl PeepholePass for OptimizeFetchValPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x)),
				// Instruction::ScaleAndMoveVal {
				// 	offset: Offset::Relative(y),
				// 	factor,
				// },
				Instruction::Super(SuperInstruction::ScaleAnd {
					offset: Offset::Relative(y),
					factor,
					action: ScaleAnd::Move,
				}),
				Instruction::MovePtr(Offset::Relative(z)),
			] if *y == *z && -x == *y => Some(Change::ReplaceOne(Instruction::fetch_and_scale_val(
				*factor, x,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(y),
					..
				}),
				Instruction::MovePtr(Offset::Relative(z))
			]
			if *y == *z && -x == *y
		)
	}
}
