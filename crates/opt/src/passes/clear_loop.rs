use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeClearLoopPass;

impl LoopPass for OptimizeClearLoopPass {
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
			] => Some(Change::swap([
				Instruction::move_ptr(*x),
				Instruction::clear_val(),
				Instruction::move_ptr(*y),
				Instruction::clear_val(),
			])),
			[
				Instruction::SetVal {
					offset: Some(offset),
					value: None,
				},
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::swap([
				Instruction::clear_val_at(*offset),
				Instruction::clear_val(),
			])),
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::SetVal {
					value: None,
					offset: Some(offset),
				},
			] => Some(Change::swap([
				Instruction::clear_val(),
				Instruction::clear_val_at(*offset),
			])),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(4))
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
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::SetVal {
					offset: Some(..),
					value: None,
				}
			] | [
				Instruction::SetVal {
					offset: Some(..),
					value: None,
				},
				Instruction::IncVal {
					value: -1,
					offset: None
				}
			]
		)
	}
}
