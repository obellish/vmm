use vmm_ir::{Instruction, WriteInstruction};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeWriteAndSetPass;

impl PeepholePass for OptimizeWriteAndSetPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Write(WriteInstruction::Cell { count, offset: x }),
				Instruction::SetVal { value, offset: y },
			] if *x == *y => Some(Change::replace(Instruction::write_many_and_set_at(
				*count,
				x,
				value.get_or_zero(),
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Write(WriteInstruction::Cell { offset: x, .. }),
				Instruction::SetVal { offset: y, .. }
			]
			if *x == *y
		)
	}
}
