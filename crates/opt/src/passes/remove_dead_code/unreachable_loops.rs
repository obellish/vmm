use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnreachableLoopsPass;

impl PeepholePass for RemoveUnreachableLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					offset: None,
					value: None,
				},
				Instruction::DynamicLoop(..),
			] => Some(Change::ReplaceOne(Instruction::clear_val())),
			[
				Instruction::ScaleAndMoveVal { offset, factor },
				Instruction::DynamicLoop(..),
			] => Some(Change::ReplaceOne(Instruction::scale_and_move_val(
				*offset, *factor,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					value: None,
					offset: None
				} | Instruction::ScaleAndMoveVal { .. },
				Instruction::DynamicLoop(..)
			]
		)
	}
}
