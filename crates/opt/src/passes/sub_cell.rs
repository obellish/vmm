use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeSubCellPass;

impl LoopPass for OptimizeSubCellPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::IncVal {
					value: -1,
					offset: Some(offset),
				},
			]
			| [
				Instruction::IncVal {
					value: -1,
					offset: Some(offset),
				},
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::replace(Instruction::sub_cell(offset))),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::IncVal {
					value: -1,
					offset: Some(..)
				}
			] | [
				Instruction::IncVal {
					value: -1,
					offset: Some(..)
				},
				Instruction::IncVal {
					value: -1,
					offset: None
				}
			]
		)
	}
}
