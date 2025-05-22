use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CombineMoveChangePass;

impl PeepholePass for CombineMoveChangePass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(Offset::Relative(x @ 1..=isize::MAX)),
				Instruction::IncVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y @ isize::MIN..=0)),
			] => Some(Change::Replace(vec![
				Instruction::IncVal {
					value: *value,
					offset: Some(Offset::Relative(*x)),
				},
				Instruction::move_ptr_by(*x + *y),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(1..=isize::MAX)),
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(isize::MIN..=0))
			]
		)
	}
}
