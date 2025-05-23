use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct MoveValToFetchPass;

impl PeepholePass for MoveValToFetchPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::MoveAndAddVal {
					offset: Offset::Relative(y),
					factor,
				},
				Instruction::MovePtr(Offset::Relative(z)),
			] if *y == *z && -x == *y => Some(Change::ReplaceOne(Instruction::FetchAndAddVal {
				offset: Offset::Relative(*x),
				factor: *factor,
			})),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::MoveAndAddVal {
					offset: Offset::Relative(y),
					..
				},
				Instruction::MovePtr(Offset::Relative(z))
			]
			if *y == *z && -x == *y
		)
	}
}
