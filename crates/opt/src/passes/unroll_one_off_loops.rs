use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct UnrollOneOffLoopsPass;

impl LoopPass for UnrollOneOffLoopsPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				rest @ ..,
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] if !rest.iter().any(Instruction::has_side_effect) => Some(Change::Replace(rest.to_vec())),
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				rest @ ..,
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			]
			if !rest.iter().any(Instruction::has_side_effect)
		)
	}
}
