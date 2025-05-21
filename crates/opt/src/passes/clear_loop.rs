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
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			]
			| [
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
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

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::MovePtr(_),
				Instruction::SetVal(0),
				Instruction::MovePtr(_),
				Instruction::IncVal {
					value: -1,
					offset: None
				}
			] | [
				Instruction::MovePtr(_),
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::MovePtr(_),
				Instruction::SetVal(0)
			]
		)
	}
}
