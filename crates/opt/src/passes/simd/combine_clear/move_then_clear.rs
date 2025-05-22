use vmm_ir::{Instruction, Offset, Simc};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CombineMoveAndClearToSimdPass;

impl PeepholePass for CombineMoveAndClearToSimdPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: None,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(1)),
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::Simc(Simc::Clear {
					count: 2,
					offset: None,
				}),
				Instruction::MovePtr(Offset::Relative(1)),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					value: None,
					offset: None
				},
				Instruction::MovePtr(Offset::Relative(1)),
				Instruction::SetVal {
					value: None,
					offset: None
				}
			]
		)
	}
}
