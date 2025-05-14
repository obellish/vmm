use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantMovesPass;

impl PeepholePass for RemoveRedundantMovesPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal(x),
				Instruction::MoveVal {
					offset,
					factor: multiplier,
				},
			] => Some(Change::Replace(vec![
				Instruction::SetVal(0),
				Instruction::MovePtr(*offset),
				Instruction::SetVal(x.wrapping_mul(*multiplier)),
				Instruction::MovePtr(-*offset),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::SetVal(_), Instruction::MoveVal { .. }]
		)
	}
}
