use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeConstantShiftPass;

impl PeepholePass for OptimizeConstantShiftPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value,
					offset: Offset(0),
				},
				Instruction::TakeVal(offset),
			] => Some(Change::swap([
				Instruction::clear_val(),
				Instruction::move_ptr(offset),
				Instruction::inc_val(value.get_or_zero() as i8),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					offset: Offset(0),
					..
				},
				Instruction::MoveVal(..) | Instruction::TakeVal(..)
			]
		)
	}
}
