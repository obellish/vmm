use vmm_ir::{Instruction, Offset};

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
			] if *x == -y => Some(Change::ReplaceOne(Instruction::IncVal {
				value: *value,
				offset: Some(Offset::Relative(*x)),
			})),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(y))
			]
			if *x == -y
		)
	}
}
