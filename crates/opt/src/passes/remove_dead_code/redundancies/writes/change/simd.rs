use vmm_ir::{Instruction, SimdInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantSimdChangeValBasicPass;

impl PeepholePass for RemoveRedundantSimdChangeValBasicPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Simd(SimdInstruction::SetVals { value: a, offsets }),
				Instruction::SetVal { value: b, offset },
			] if *a == *b && offsets.contains(offset) => Some(Change::remove_offset(1)),
			[
				Instruction::Simd(a @ SimdInstruction::SetVals { .. }),
				Instruction::Simd(b @ SimdInstruction::SetVals { .. }),
			] if a == b => Some(Change::remove_offset(1)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Simd(SimdInstruction::SetVals { value: a, offsets }),
				Instruction::SetVal { value: b, offset }
			]
			if *a == *b && offsets.contains(offset)
		) || matches!(
			window,
			[
				Instruction::Simd(a @ SimdInstruction::SetVals { .. }),
				Instruction::Simd(b @ SimdInstruction::SetVals { .. }),
			]
			if a == b
		)
	}
}
