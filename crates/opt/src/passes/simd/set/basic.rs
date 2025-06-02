use vmm_ir::{Instruction, SimdInstruction};
use vmm_utils::GetOrZero as _;

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
					offset: x,
				},
				Instruction::SetVal {
					value: b,
					offset: y,
				},
			] if *a == *b && *x != *y => Some(Change::Replace(Instruction::simd_set_vals(
				a.get_or_zero(),
				{
					let mut offsets = vec![*x, *y];
					offsets.sort();
					offsets
				},
			))),
			[
				Instruction::Simd(SimdInstruction::SetVals { value: a, offsets }),
				Instruction::SetVal { value: b, offset },
			] if *a == *b && !offsets.contains(offset) => Some(Change::Replace(Instruction::simd_set_vals(
				a.get_or_zero(),
				{
					let mut offsets = offsets.to_owned();
					offsets.push(*offset);
					offsets.sort();
					offsets
				},
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
					offset: x
				},
				Instruction::SetVal {
					value: b,
					offset: y
				}
			]
			if *a == *b && *x != *y
		) || matches!(
			window,
			[
				Instruction::Simd(SimdInstruction::SetVals { value: a, offsets }),
				Instruction::SetVal { value: b, offset }
			]
			if *a == *b && !offsets.contains(offset)
		)
	}
}
