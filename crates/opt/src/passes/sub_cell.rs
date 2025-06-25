use vmm_ir::{Instruction, Offset};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeSubCellPass;

impl LoopPass for OptimizeSubCellPass {
	#[inline]
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
				},
				Instruction::IncVal { value: -1, offset },
			]
			| [
				Instruction::IncVal { value: -1, offset },
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
				},
			] => Some(Change::replace(Instruction::sub_cell(offset))),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}

	#[inline]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: Offset(0)
				},
				Instruction::IncVal { value: -1, .. }
			] | [
				Instruction::IncVal { value: -1, .. },
				Instruction::IncVal {
					value: -1,
					offset: Offset(0)
				}
			]
		)
	}
}
