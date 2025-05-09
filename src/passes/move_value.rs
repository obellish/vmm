use std::cmp;

use crate::{Change, Instruction, LoopPass, StackedInstruction};

#[derive(Debug, Default, Clone, Copy)]
pub struct MoveValuePass;

impl LoopPass for MoveValuePass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::Stacked(StackedInstruction::IncVal(i)),
				Instruction::Stacked(StackedInstruction::MovePtr(x)),
				Instruction::Stacked(StackedInstruction::IncVal(j)),
				Instruction::Stacked(StackedInstruction::MovePtr(y)),
			]
			| [
				Instruction::Stacked(StackedInstruction::MovePtr(x)),
				Instruction::Stacked(StackedInstruction::IncVal(i)),
				Instruction::Stacked(StackedInstruction::MovePtr(y)),
				Instruction::Stacked(StackedInstruction::IncVal(j)),
			] if *x == -*y => {
				let i = *i;
				let j = *j;
				let x = *x;
				let y = *y;

				let min = cmp::min(i, j);
				if !matches!(min, -1) {
					return None;
				}

				let multiplier = cmp::max(i, j);

				if multiplier < 0 {
					return None;
				}

				Some(Change::ReplaceOne(Instruction::MoveVal {
					offset: x,
					multiplier: multiplier as u8,
				}))
			}
			_ => None,
		}
	}
}
