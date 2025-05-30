use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};
use vmm_wrap::Wrapping;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSetScaleValPass;

impl PeepholePass for OptimizeSetScaleValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: Some(value),
					offset: None,
				},
				Instruction::ScaleVal { factor },
			] => Some(Change::ReplaceOne(Instruction::set_val(Wrapping::mul(
				value.get(),
				*factor,
			)))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					value: Some(..),
					offset: None
				},
				Instruction::ScaleVal { .. }
			]
		)
	}
}
