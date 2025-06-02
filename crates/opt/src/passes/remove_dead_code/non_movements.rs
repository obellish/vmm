use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

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
			] => Some(Change::replace(Instruction::set_val(value.get_or_zero()))),
			[
				Instruction::IncVal {
					value,
					offset: Some(Offset::Relative(0)),
				},
			] => Some(Change::replace(Instruction::inc_val(*value))),
			[
				Instruction::Write {
					offset: Some(Offset::Relative(0)),
					count,
				},
			] => Some(Change::replace(Instruction::write_many(*count))),
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
			} | Instruction::Write {
				offset: Some(Offset::Relative(0)),
				..
			}]
		)
	}
}
