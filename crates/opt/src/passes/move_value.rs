use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct MoveValuePass;

impl LoopPass for MoveValuePass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal(-1),
				Instruction::MovePtr(x),
				Instruction::IncVal(j),
				Instruction::MovePtr(y),
			]
			| [
				Instruction::IncVal(j),
				Instruction::MovePtr(x),
				Instruction::IncVal(-1),
				Instruction::MovePtr(y),
			] if *x == -y => {
				let j = *j;
				let x = *x;

				if j < 0 {
					return None;
				}

				Some(Change::ReplaceOne(Instruction::MoveVal {
					offset: x,
					multiplier: j as u8,
				}))
			}
			_ => None,
		}
	}
}
