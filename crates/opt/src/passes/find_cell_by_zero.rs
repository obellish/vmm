use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeFindCellByZeroPass;

impl PeepholePass for OptimizeFindCellByZeroPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::FindZero(jump_by), Instruction::MovePtr(offset)] => Some(
				Change::Replace(Instruction::find_cell_by_zero(jump_by, offset)),
			),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::FindZero(..), Instruction::MovePtr(..)]
		)
	}
}
