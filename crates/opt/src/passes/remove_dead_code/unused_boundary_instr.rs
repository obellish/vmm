use vmm_ir::{BlockInstruction, Instruction, Offset, Value};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnusedBoundaryInstrPass;

impl PeepholePass for RemoveUnusedBoundaryInstrPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Boundary,
				Instruction::Block(BlockInstruction::DynamicLoop(..))
				| Instruction::SetVal { value: None, .. },
			] => Some(Change::remove_offset(1)),
			[
				Instruction::Boundary,
				Instruction::IncVal {
					value: Value::Constant(value),
					offset: Offset(offset),
				},
			] => Some(Change::swap([
				Instruction::Boundary,
				Instruction::set_val_at(*value as u8, offset),
			])),
			[
				Instruction::MovePtr(..) | Instruction::SetVal { .. },
				Instruction::Boundary,
			] => Some(Change::remove_offset(0)),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Boundary,
				Instruction::Block(BlockInstruction::DynamicLoop(..))
					| Instruction::SetVal { value: None, .. }
					| Instruction::IncVal { .. }
			] | [
				Instruction::MovePtr(..) | Instruction::SetVal { .. },
				Instruction::Boundary
			]
		)
	}
}
