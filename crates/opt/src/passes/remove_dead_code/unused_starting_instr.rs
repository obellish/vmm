use vmm_ir::{BlockInstruction, Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnusedStartingInstrPass;

impl PeepholePass for RemoveUnusedStartingInstrPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Start,
				Instruction::Block(BlockInstruction::DynamicLoop(..))
				| Instruction::SetVal { value: None, .. },
			] => Some(Change::remove_offset(1)),
			[
				Instruction::Start,
				Instruction::IncVal {
					value,
					offset: Offset(offset),
				},
			] => Some(Change::swap([
				Instruction::Start,
				Instruction::set_val_at(*value as u8, offset),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Start,
				Instruction::Block(BlockInstruction::DynamicLoop(..))
					| Instruction::SetVal { value: None, .. }
					| Instruction::IncVal { .. }
			]
		)
	}
}
