use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CombineMoveChangePass;

impl PeepholePass for CombineMoveChangePass {
	const SIZE: usize = 3;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset(x @ 1..=isize::MAX)),
				Instruction::IncVal {
					value,
					offset: Offset(0),
				},
				Instruction::MovePtr(Offset(y @ isize::MIN..=0)),
			] => Some(Change::swap([
				Instruction::inc_val_at(*value, x),
				Instruction::move_ptr_by(*x + *y),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset(1..=isize::MAX)),
				Instruction::IncVal {
					offset: Offset(0),
					..
				},
				Instruction::MovePtr(Offset(isize::MIN..=0))
			]
		)
	}
}
