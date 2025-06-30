use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantShiftsPass;

impl PeepholePass for RemoveRedundantShiftsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::TakeVal(x), Instruction::MoveVal(y)] => Some(Change::swap([
				Instruction::move_val(x + y),
				Instruction::move_ptr(x),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::TakeVal(..) | Instruction::MoveVal(..),
				Instruction::MoveVal(..)
			]
		)
	}
}
