use std::num::NonZeroU8;

use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnusedStartingInstrPass;

impl PeepholePass for RemoveUnusedStartingInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Start,
				Instruction::DynamicLoop(_) | Instruction::SetVal { value: None, .. },
			] => Some(Change::ReplaceOne(Instruction::Start)),
			[Instruction::Start, Instruction::IncVal { value, offset }] => {
				Some(Change::Replace(vec![
					Instruction::Start,
					Instruction::SetVal {
						value: NonZeroU8::new(*value as u8),
						offset: *offset,
					},
				]))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Start,
				Instruction::DynamicLoop(_)
					| Instruction::SetVal { value: None, .. }
					| Instruction::IncVal { .. }
			]
		)
	}
}
