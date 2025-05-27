use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveNonMovementOffsetsPass;

impl PeepholePass for RemoveNonMovementOffsetsPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value,
					offset: Some(Offset::Relative(0)),
				},
			] => Some(Change::ReplaceOne(Instruction::SetVal {
				value: *value,
				offset: None,
			})),
			[
				Instruction::IncVal {
					value,
					offset: Some(Offset::Relative(0)),
				},
			] => Some(Change::ReplaceOne(Instruction::inc_val(*value))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::IncVal {
				offset: Some(Offset::Relative(0)),
				..
			} | Instruction::SetVal {
				offset: Some(Offset::Relative(0)),
				..
			}]
		)
	}
}
