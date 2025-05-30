use vmm_ir::{Instruction, SimdInstruction};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeSimdSetLoopPass;

impl LoopPass for OptimizeSimdSetLoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
        None
    }

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[Instruction::Simd(SimdInstruction::SetVals {
				value: None,
				offsets
			})]
            if offsets.contains(&None)
		)
	}
}
