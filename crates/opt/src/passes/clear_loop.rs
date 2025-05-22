use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct ClearLoopPass;

impl LoopPass for ClearLoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::MovePtr(x),
				Instruction::SetVal {
					value: None,
					offset: None,
				},
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
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::MovePtr(*x),
				Instruction::clear_val(),
				Instruction::MovePtr(*y),
				Instruction::clear_val(),
			])),
			[
				Instruction::SetVal {
					offset: offset @ Some(_),
					value: None,
				},
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::SetVal {
					value: None,
					offset: *offset,
				},
				Instruction::clear_val(),
			])),
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::MovePtr(_),
				Instruction::SetVal {
					value: None,
					offset: None
				},
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
				Instruction::SetVal {
					value: None,
					offset: None
				}
			] | [
				Instruction::SetVal {
					offset: Some(_),
					value: None
				},
				Instruction::IncVal {
					offset: None,
					value: -1
				}
			]
		)
	}
}
