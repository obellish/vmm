use vmm_ir::Instruction;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSetZeroPass;

impl PeepholePass for OptimizeSetZeroPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::FindZero(offset),
				Instruction::IncVal {
					value,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::find_and_set_zero(
				*value as u8,
				*offset,
			))),
			[
				Instruction::FindZero(offset),
				Instruction::SetVal {
					value,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::find_and_set_zero(
				value.get_or_zero(),
				*offset,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::FindZero(..),
				Instruction::IncVal { offset: None, .. } | Instruction::SetVal { offset: None, .. }
			]
		)
	}
}
