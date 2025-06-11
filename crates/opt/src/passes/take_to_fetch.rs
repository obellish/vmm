use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeTakeFetchValPass;

impl PeepholePass for OptimizeTakeFetchValPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::MovePtr(x), Instruction::TakeVal(y)] if *x == -y => {
				Some(Change::replace(Instruction::fetch_val(*x)))
			}
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::MovePtr(x), Instruction::TakeVal(y)] if *x == -y)
	}
}
