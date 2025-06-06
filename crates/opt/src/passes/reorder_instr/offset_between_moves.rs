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
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(y),
			] => Some(Change::swap([
				Instruction::inc_val_at(*value, *x),
				Instruction::move_ptr(*x + *y),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::SetVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(y),
			] => Some(Change::swap([
				Instruction::set_val_at(value.get_or_zero(), *x),
				Instruction::move_ptr(*x + *y),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::Write {
					offset: None,
					count,
				},
				Instruction::MovePtr(y),
			] => Some(Change::swap([
				Instruction::write_many_at(*count, *x),
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
					| Instruction::Write { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(_))
			]
		)
	}
}
