use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeFetchValPass;

impl PeepholePass for OptimizeFetchValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::MoveVal(Offset::Relative(y)),
			] if *x == -y => Some(Change::Replace(vec![
				Instruction::fetch_val(*x),
				Instruction::move_ptr(*x),
				Instruction::clear_val(),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::MoveVal(Offset::Relative(y))
			]
			if *x == -y
		)
	}
}
