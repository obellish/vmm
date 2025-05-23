use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderChangeBetweenMovesPass;

impl PeepholePass for ReorderChangeBetweenMovesPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y)),
			] => Some(Change::Replace(vec![
				Instruction::inc_val_at(*value, x),
				Instruction::move_ptr(*x + *y),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(_)),
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(_))
			]
		)
	}
}
