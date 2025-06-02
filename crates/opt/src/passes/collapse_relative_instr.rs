use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseRelativeInstrPass;

impl PeepholePass for CollapseRelativeInstrPass {
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
			] if *x == -y => Some(Change::Replace(Instruction::inc_val_at(*value, x))),
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::SetVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y)),
			] if *x == -y => Some(Change::Replace(Instruction::set_val_at(
				value.get_or_zero(),
				*x,
			))),
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::Write {
					offset: None,
					count,
				},
				Instruction::MovePtr(Offset::Relative(y)),
			] if *x == -y => Some(Change::Replace(Instruction::write_many_at(*count, *x))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal { offset: None, .. }
					| Instruction::SetVal { offset: None, .. }
					| Instruction::Write { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(y))
			]
			if *x == -y
		)
	}
}
