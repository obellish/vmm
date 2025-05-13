use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct ClearLoopPass;

impl LoopPass for ClearLoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::MovePtr(x),
				Instruction::SetVal(0),
				Instruction::MovePtr(y),
				Instruction::IncVal(-1),
			]
			| [
				Instruction::MovePtr(x),
				Instruction::IncVal(-1),
				Instruction::MovePtr(y),
				Instruction::SetVal(0),
			] => Some(Change::Replace(vec![
				Instruction::MovePtr(*x),
				Instruction::SetVal(0),
				Instruction::MovePtr(*y),
				Instruction::SetVal(0),
			])),
			_ => None,
		}
	}
}
