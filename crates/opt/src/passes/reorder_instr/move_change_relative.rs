use vmm_ir::Instruction;
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
			] if *x == -y => Some(Change::swap([
				Instruction::inc_val(*value),
				Instruction::move_ptr(*x),
			])),
			[
				Instruction::MovePtr(x),
				Instruction::SetVal { value, offset: y },
			] if *x == -y => Some(Change::swap([
				Instruction::set_val(value.get_or_zero()),
				Instruction::move_ptr(*x),
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
				Instruction::IncVal {
					offset: y,
					..
				} | Instruction::SetVal {
					offset: y,
					..
				}
			]
			if *x == -y
		)
	}
}
