use vmm_ir::{Instruction, SimdInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSimdIncInstrPass;

impl PeepholePass for OptimizeSimdIncInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: a,
					offset: x,
				},
				Instruction::IncVal {
					value: b,
					offset: y,
				},
			] if *a == *b && *x != *y => Some(Change::ReplaceOne(Instruction::simd_inc_vals(*a, {
				let mut offsets = vec![*x, *y];
				offsets.sort();
				offsets
			}))),
			[
				Instruction::Simd(SimdInstruction::IncVals { value: a, offsets }),
				Instruction::IncVal { value: b, offset },
			] if *a == *b && !offsets.contains(offset) => {
				Some(Change::ReplaceOne(Instruction::simd_inc_vals(*a, {
					let mut offsets = offsets.to_owned();
					offsets.push(*offset);
					offsets.sort();
					offsets
				})))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal {
					value: a,
					offset: x
				},
				Instruction::IncVal {
					value: b,
					offset: y
				}
			]
			if *a == *b && *x != *y
		) || matches!(
			window,
			[
				Instruction::Simd(SimdInstruction::IncVals { value: a, offsets }),
				Instruction::IncVal { value: b, offset }
			]
			if *a == *b && !offsets.contains(offset)
		)
	}
}
