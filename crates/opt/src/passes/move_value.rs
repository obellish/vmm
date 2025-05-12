use std::cmp;

use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct MoveValuePass;

impl LoopPass for MoveValuePass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal(i),
				Instruction::MovePtr(x),
				Instruction::IncVal(j),
				Instruction::MovePtr(y),
			]
			| [
				Instruction::MovePtr(x),
				Instruction::IncVal(i),
				Instruction::MovePtr(y),
				Instruction::IncVal(j),
			] if *x == -*y => {
				let i = *i;
				let j = *j;
				let x = *x;

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
