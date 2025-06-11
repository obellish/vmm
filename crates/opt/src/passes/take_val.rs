use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeTakeValPass;

impl PeepholePass for OptimizeTakeValPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::MoveVal(x), Instruction::MovePtr(y)] if *x == *y => {
				Some(Change::replace(Instruction::take_val(*x)))
			}
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::MoveVal(x), Instruction::MovePtr(y)] if *x == *y)
	}
}
