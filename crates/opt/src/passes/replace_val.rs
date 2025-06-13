use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeReplaceValPass;

impl PeepholePass for OptimizeReplaceValPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
				Instruction::FetchVal(offset),
			] => Some(Change::replace(Instruction::replace_val(offset))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					value: None,
					offset: Offset(0)
				},
				Instruction::FetchVal(..)
			]
		)
	}
}
