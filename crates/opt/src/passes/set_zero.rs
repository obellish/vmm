use std::num::NonZeroU8;

use vmm_ir::{Instruction, Offset};

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
					offset: Offset(0),
				},
			] => Some(Change::replace(Instruction::find_and_set_zero(
				NonZeroU8::new(*value as u8)?,
				*offset,
			))),
			[
				Instruction::FindZero(offset),
				Instruction::SetVal {
					value: Some(value),
					offset: Offset(0),
				},
			] => Some(Change::replace(Instruction::find_and_set_zero(
				*value, *offset,
			))),
			[
				Instruction::FindZero(..),
				Instruction::SetVal {
					offset: Offset(0),
					value: None,
				},
			] => Some(Change::remove_offset(1)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::FindZero(..),
				Instruction::IncVal {
					offset: Offset(0),
					..
				} | Instruction::SetVal {
					offset: Offset(0),
					..
				}
			]
		)
	}
}
