use vmm_ir::{Instruction, Offset, SimdInstruction};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeDupeAndScaleValPass;

impl LoopPass for OptimizeDupeAndScaleValPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		let [
			Instruction::IncVal {
				value: -1,
				offset: None,
			},
			Instruction::Simd(SimdInstruction::IncVals { value, offsets }),
		] = loop_values
		else {
			return None;
		};

		let value = *value as u8;

		let mut output = vec![Instruction::dupe_val(
			offsets.iter().copied().flatten().collect(),
		)];

		for offset in offsets.iter().copied().flatten().filter_map(|offset| {
			if let Offset::Relative(offset) = offset {
				Some(offset)
			} else {
				None
			}
		}) {
			output.extend([
				Instruction::move_ptr(offset),
				Instruction::scale_val(value),
				Instruction::move_ptr(-offset),
			]);
		}

		Some(Change::swap(output))
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					offset: None,
					value: -1
				},
				Instruction::Simd(SimdInstruction::IncVals {
					offsets,
					value
				})
			]
			if !offsets.contains(&None) && !matches!(value, 1)
		)
	}
}
