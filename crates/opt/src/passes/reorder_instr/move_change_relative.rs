use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderMoveChangePass;

impl PeepholePass for ReorderMoveChangePass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal {
					value,
					offset: Some(Offset::Relative(y)),
				},
			] if *x == -y => Some(Change::Replace(vec![
				Instruction::inc_val(*value),
				Instruction::move_ptr_by(*x),
			])),
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::SetVal {
					value,
					offset: Some(Offset::Relative(y)),
				},
			] if *x == -y => Some(Change::Replace(vec![
				Instruction::set_val(value.get_or_zero()),
				Instruction::move_ptr_by(*x),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					..
				} | Instruction::SetVal {
					offset: Some(Offset::Relative(y)),
					..
				}
			]
			if *x == -y
		)
	}
}
