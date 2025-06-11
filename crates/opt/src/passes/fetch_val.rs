use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeFetchValPass;

impl PeepholePass for OptimizeFetchValPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::MovePtr(x), Instruction::MoveVal(y)] if *x == -y => Some(Change::swap([
				Instruction::fetch_val(*x),
				Instruction::move_ptr(*x),
				Instruction::clear_val(),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(x),
				Instruction::MoveVal(y)
			]
			if *x == -y
		)
	}
}
