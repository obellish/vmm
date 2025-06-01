use vmm_ir::{Instruction, LoopInstruction, Offset, SimdInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnusedStartingInstrPass;

impl PeepholePass for RemoveUnusedStartingInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Start,
				Instruction::Loop(LoopInstruction::Dynamic(..))
				| Instruction::SetVal { value: None, .. },
			] => Some(Change::RemoveOffset(1)),
			[
				Instruction::Start,
				Instruction::IncVal {
					value,
					offset: Some(Offset::Relative(offset)),
				},
			] => Some(Change::Replace(vec![
				Instruction::Start,
				Instruction::set_val_at(*value as u8, offset),
			])),
			[
				Instruction::Start,
				Instruction::IncVal {
					value,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::Start,
				Instruction::set_val(*value as u8),
			])),
			[
				Instruction::Start,
				Instruction::Simd(SimdInstruction::IncVals { value, offsets }),
			] => Some(Change::Replace(vec![
				Instruction::Start,
				Instruction::simd_set_vals(*value as u8, offsets.to_owned()),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Start,
				Instruction::Loop(LoopInstruction::Dynamic(..))
					| Instruction::SetVal { value: None, .. }
					| Instruction::IncVal { .. }
					| Instruction::Simd(SimdInstruction::IncVals { .. })
			]
		)
	}
}
