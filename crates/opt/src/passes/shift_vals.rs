use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeShiftValsPass;

impl LoopPass for OptimizeShiftValsPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[Instruction::MovePtr(x), Instruction::MoveVal(y)]
			| [Instruction::MoveVal(y), Instruction::MovePtr(x)]
				if *x == -y =>
			{
				Some(Change::Replace(Instruction::shift_vals(y)))
			}
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[Instruction::MovePtr(x), Instruction::MoveVal(y)]
				| [Instruction::MoveVal(y), Instruction::MovePtr(x)]
			if *x == -y
		)
	}
}
