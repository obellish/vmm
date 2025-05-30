use vmm_ir::{Instruction, Offset, SimdInstruction};

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
					offset: Some(Offset::Relative(x)),
				},
				Instruction::IncVal {
					value: b,
					offset: Some(Offset::Relative(y)),
				},
			] if *a == *b && *x != *y => Some(Change::ReplaceOne(Instruction::simd_inc_by(
				*a,
				vec![x.into(), y.into()],
			))),
			[
				Instruction::Simd(SimdInstruction::IncBy { value: a, offsets }),
				Instruction::IncVal {
					value: b,
					offset: Some(Offset::Relative(x)),
				},
			] if *a == *b && !offsets.contains(&Offset::Relative(*x)) => {
				Some(Change::ReplaceOne(Instruction::simd_inc_by(*a, {
					let mut offsets = offsets.clone();
					offsets.push(x.into());
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
					offset: Some(Offset::Relative(x))
				},
				Instruction::IncVal {
					value: b,
					offset: Some(Offset::Relative(y))
				}
			]
			if *a == *b && *x != *y
		) || matches!(
			window,
			[
				Instruction::Simd(SimdInstruction::IncBy { value: a, offsets }),
				Instruction::IncVal {
					value: b,
					offset: Some(Offset::Relative(x))
				}
			]
			if *a == *b && !offsets.contains(&Offset::Relative(*x))
		)
	}
}
