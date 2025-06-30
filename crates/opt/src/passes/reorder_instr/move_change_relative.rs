use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderMoveChangePass;

impl PeepholePass for ReorderMoveChangePass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(x),
				Instruction::IncVal { value, offset: y },
			] if *x == -y && !matches!(x, Offset(0)) => Some(Change::swap([
				Instruction::inc_val(*value),
				Instruction::move_ptr(*x),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::SetVal { value, offset: y },
			] if *x == -y && !matches!(x, Offset(0)) => Some(Change::swap([
				Instruction::set_val(value.get_or_zero()),
				Instruction::move_ptr(*x),
			])),
			[
				Instruction::SetVal { value, offset: x },
				Instruction::MovePtr(y),
			] if *x == *y => Some(Change::swap([
				Instruction::move_ptr(x),
				Instruction::set_val(value.get_or_zero()),
			])),
			[
				Instruction::IncVal { value, offset: x },
				Instruction::MovePtr(y),
			] if *x == *y => Some(Change::swap([
				Instruction::move_ptr(x),
				Instruction::inc_val(*value),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(x),
				Instruction::IncVal { offset: y, .. } | Instruction::SetVal { offset: y, .. }
			]
			if *x == -y && !matches!(x, Offset(0))
		) || matches!(
			window,
			[
				Instruction::SetVal { offset: x, .. } | Instruction::IncVal { offset: x, .. },
				Instruction::MovePtr(y)
			]
			if *x == *y
		)
	}
}
