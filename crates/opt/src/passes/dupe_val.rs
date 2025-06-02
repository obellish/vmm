use vmm_ir::{Instruction, SimdInstruction};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeDuplicateValPass;

impl LoopPass for OptimizeDuplicateValPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::Simd(SimdInstruction::IncVals { value: 1, offsets }),
			]
			| [
				Instruction::Simd(SimdInstruction::IncVals { value: 1, offsets }),
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::Replace(Instruction::dupe_val(
				offsets.iter().copied().flatten().collect(),
			))),
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::Simd(SimdInstruction::IncVals { value: 1, .. })
			] | [
				Instruction::Simd(SimdInstruction::IncVals { value: 1, .. }),
				Instruction::IncVal {
					value: -1,
					offset: None
				}
			]
		)
	}
}
