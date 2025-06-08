use vmm_ir::{Instruction, SpanInstruction};
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSetSpanPass;

impl PeepholePass for OptimizeSetSpanPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [
			Instruction::SetVal {
				value: a,
				offset: x,
			},
			Instruction::SetVal {
				value: b,
				offset: y,
			},
		] = *window
		else {
			return false;
		};

		if a != b {
			return false;
		}

		let x = x.get_or_zero();

		let y = y.get_or_zero();

		if (x + 1) != y {
			return false;
		}

		true
	}
}
