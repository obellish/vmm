use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeZeroedCellIncValPass;

impl PeepholePass for OptimizeZeroedCellIncValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::ScaleAndMoveVal { offset, factor },
				Instruction::IncVal {
					value,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::scale_and_move_val(*offset, *factor),
				Instruction::set_val(*value as u8),
			])),
			[
				Instruction::FindZero(i),
				Instruction::IncVal {
					offset: None,
					value,
				},
			] => Some(Change::Replace(vec![
				Instruction::find_zero(*i),
				Instruction::set_val(*value as u8),
			])),
			[
				dyn_loop @ Instruction::DynamicLoop(..),
				Instruction::IncVal {
					value,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				dyn_loop.clone(),
				Instruction::set_val(*value as u8),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [i, Instruction::IncVal { offset: None, .. }] if i.is_zeroing_cell() && !i.is_clear_val())
	}
}
