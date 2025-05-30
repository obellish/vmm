use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSimdSetInstrPass;

impl PeepholePass for OptimizeSimdSetInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: a,
					offset: Some(Offset::Relative(x)),
				},
				Instruction::SetVal {
					value: b,
					offset: Some(Offset::Relative(y)),
				},
			] if *a == *b && *x != *y => Some(Change::ReplaceOne(Instruction::simd_set_vals(
				a.get_or_zero(),
				vec![Some(x.into()), Some(y.into())],
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					value: a,
					offset: Some(Offset::Relative(x))
				},
				Instruction::SetVal {
					value: b,
					offset: Some(Offset::Relative(y))
				}
			]
			if *a == *b && *x != *y
		)
	}
}
