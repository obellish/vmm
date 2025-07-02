use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderOffsetBetweenMovesPass;

impl PeepholePass for ReorderOffsetBetweenMovesPass {
	const SIZE: usize = 3;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value,
					offset: Offset(0),
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
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
			] => Some(Change::swap([
				Instruction::set_val_at(value.get_or_zero(), *x),
				Instruction::move_ptr(*x + *y),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::Write { offset: Offset(0) },
				Instruction::MovePtr(y),
			] => Some(Change::swap([
				Instruction::write_once_at(x),
				Instruction::move_ptr(*x + *y),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::Write { offset: y },
				Instruction::MovePtr(z),
			] => Some(Change::swap([
				Instruction::write_once_at(x + y),
				Instruction::move_ptr(x + z),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::IncVal { value, offset: y },
				Instruction::MovePtr(z),
			] => Some(Change::swap([
				Instruction::inc_val_at(*value, x + y),
				Instruction::move_ptr(x + z),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(..),
				Instruction::IncVal { .. } | Instruction::SetVal { .. } | Instruction::Write { .. },
				Instruction::MovePtr(..)
			]
		)
	}
}
