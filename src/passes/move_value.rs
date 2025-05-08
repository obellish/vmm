use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default, Clone, Copy)]
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
				// dbg!((i, j));

				let min = std::cmp::min(*i, *j);
				if min != -1 {
					return None;
				}
				let multiplier = std::cmp::max(*i, *j);

				if multiplier < 0 {
					return None;
				}

				// None
				Some(Change::ReplaceOne(Instruction::MoveVal {
					offset: *x,
					multiplier: multiplier as u8,
				}))
			}
			_ => None,
		}
	}
}
