use vmm_ir::{Instruction, PtrMovement as _};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct RemoveInfiniteLoopsPass;

impl LoopPass for RemoveInfiniteLoopsPass {
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::remove())
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				..,
				Instruction::Read
					| Instruction::SetVal {
						value: Some(..),
						offset: None
					}
			]
		) && !loop_values.might_move_ptr()
	}
}
