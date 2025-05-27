use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderOffsetBetweenMovesPass;

impl PeepholePass for ReorderOffsetBetweenMovesPass {
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
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::SetVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y)),
			] => Some(Change::Replace(vec![
				Instruction::set_val_at(value.get_or_zero(), x),
				Instruction::move_ptr(*x + *y),
			])),
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::Write { offset: None },
				Instruction::MovePtr(Offset::Relative(y)),
			] => Some(Change::Replace(vec![
				Instruction::write_at(x),
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
				Instruction::IncVal { offset: None, .. }
					| Instruction::SetVal { offset: None, .. }
					| Instruction::Write { offset: None },
				Instruction::MovePtr(Offset::Relative(_))
			]
		)
	}
}
