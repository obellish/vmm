use vmm_ir::{Instruction, SimdInstruction};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeSubCellPass;

impl LoopPass for OptimizeSubCellPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[Instruction::Simd(SimdInstruction::IncVals { value: -1, offsets })]
				if matches!(offsets.len(), 2) && offsets.contains(&None) =>
			{
				let ((Some(offset), None) | (None, Some(offset))) = (offsets[0], offsets[1]) else {
					return None;
				};

				Some(Change::ReplaceOne(Instruction::sub_cell(offset)))
			}
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[Instruction::Simd(SimdInstruction::IncVals {
				value: -1,
				offsets
			})]
			if matches!(offsets.len(), 2) && offsets.contains(&None)
		)
	}
}
