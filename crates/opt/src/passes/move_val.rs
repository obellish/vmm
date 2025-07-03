use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeMoveValPass;

impl PeepholePass for OptimizeMoveValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::TakeVal(x), Instruction::MovePtr(y)] => Some(Change::swap([
				Instruction::move_val(x),
				Instruction::move_ptr(x + y),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::TakeVal(..), Instruction::MovePtr(..)])
	}
}
