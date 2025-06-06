use vmm_ir::Instruction;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseRelativeInstrPass;

impl PeepholePass for CollapseRelativeInstrPass {
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
			] if *x == -y => Some(Change::replace(Instruction::inc_val_at(*value, x))),
			[
				Instruction::MovePtr(x),
				Instruction::SetVal {
					value,
					offset: None,
				},
				Instruction::MovePtr(y),
			] if *x == -y => Some(Change::replace(Instruction::set_val_at(
				value.get_or_zero(),
				*x,
			))),
			[
				Instruction::MovePtr(x),
				Instruction::Write {
					offset: None,
					count,
				},
				Instruction::MovePtr(y),
			] if *x == -y => Some(Change::replace(Instruction::write_many_at(*count, *x))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(x),
				Instruction::IncVal { offset: None, .. }
					| Instruction::SetVal { offset: None, .. }
					| Instruction::Write { offset: None, .. },
				Instruction::MovePtr(y)
			]
			if *x == -y
		)
	}
}
