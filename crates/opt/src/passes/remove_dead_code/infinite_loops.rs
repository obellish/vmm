use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct RemoveInfiniteLoopsPass;

impl LoopPass for RemoveInfiniteLoopsPass {
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::Remove)
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
		)
	}
}
