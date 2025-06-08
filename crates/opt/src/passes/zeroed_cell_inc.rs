use vmm_ir::{Instruction, IsZeroingCell as _, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeZeroedCellIncValPass;

impl PeepholePass for OptimizeZeroedCellIncValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				i,
				Instruction::IncVal {
					value,
					offset: Offset(0),
				},
			] if i.is_zeroing_cell() => Some(Change::swap([
				i.clone(),
				Instruction::set_val(*value as u8),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [i, Instruction::IncVal { offset: Offset(0), .. }] if i.is_zeroing_cell())
	}
}
