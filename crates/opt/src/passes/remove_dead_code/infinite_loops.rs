use vmm_ir::{Instruction, Offset, PtrMovement as _};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct RemoveInfiniteLoopsPass;

impl LoopPass for RemoveInfiniteLoopsPass {
	#[inline]
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::remove())
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(1, None)
	}

	#[inline]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				..,
				Instruction::Read
					| Instruction::SetVal {
						value: Some(..),
						offset: Offset(0)
					}
			]
		) && !loop_values.might_move_ptr()
	}
}
