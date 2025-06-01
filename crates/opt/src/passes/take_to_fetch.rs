use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeTakeFetchValPass;

impl PeepholePass for OptimizeTakeFetchValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::TakeVal(Offset::Relative(y)),
			] if *x == -y => Some(Change::ReplaceOne(Instruction::fetch_val(*x))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::MovePtr(Offset::Relative(x)), Instruction::TakeVal(Offset::Relative(y))] if *x == -y)
	}
}
