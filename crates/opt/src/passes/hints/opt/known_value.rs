use vmm_ir::{CompilerHint, Instruction};
use vmm_num::ops::WrappingAdd;
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeKnownValueHintPass;

impl PeepholePass for OptimizeKnownValueHintPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Hint(CompilerHint::KnownValue {
					value: a,
					offset: x,
				}),
				Instruction::IncVal {
					value: b,
					offset: y,
				},
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(
				WrappingAdd::wrapping_add(a.get_or_zero(), b),
				x,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Hint(CompilerHint::KnownValue { offset: x, .. }),
				Instruction::IncVal { offset: y, .. }
			]
			if *x == *y
		)
	}
}
