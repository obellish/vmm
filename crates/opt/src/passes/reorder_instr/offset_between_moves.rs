use vmm_ir::{Instruction, Offset, Value};
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
					value: Value::Constant(value),
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
				Instruction::Write {
					value: Value::CellAt(Offset(0)),
				},
				Instruction::MovePtr(y),
			] => Some(Change::swap([
				Instruction::write_once_at(x),
				Instruction::move_ptr(*x + *y),
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
				Instruction::IncVal {
					value: Value::Constant(..),
					offset: Offset(0)
				} | Instruction::SetVal {
					offset: Offset(0),
					..
				} | Instruction::Write {
					value: Value::CellAt(Offset(0))
				},
				Instruction::MovePtr(..)
			]
		)
	}
}
