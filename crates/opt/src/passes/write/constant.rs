use vmm_ir::{Instruction, WriteInstruction};
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeWriteConstPass;

impl PeepholePass for OptimizeWriteConstPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal { value, offset: x },
				Instruction::Write(WriteInstruction::Cell {
					count: 1,
					offset: y,
				}),
			] if *x == *y => Some(Change::replace(Instruction::write_byte(
				value.get_or_zero(),
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: x, .. },
				Instruction::Write(WriteInstruction::Cell { offset: y, .. })
			]
			if *x == *y
		)
	}
}
