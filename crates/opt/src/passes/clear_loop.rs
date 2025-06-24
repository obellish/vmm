use vmm_ir::{Instruction, Offset, Value};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeClearLoopPass;

impl LoopPass for OptimizeClearLoopPass {
	#[inline]
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::MovePtr(x),
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0),
				},
			]
			| [
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
			] => Some(Change::swap([
				Instruction::move_ptr(*x),
				Instruction::clear_val(),
				Instruction::move_ptr(*y),
				Instruction::clear_val(),
			])),
			[
				Instruction::SetVal {
					offset,
					value: None,
				},
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0),
				},
			] => Some(Change::swap([
				Instruction::clear_val_at(*offset),
				Instruction::clear_val(),
			])),
			[
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0),
				},
				Instruction::SetVal {
					value: None,
					offset,
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

	#[inline]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::MovePtr(_),
				Instruction::SetVal {
					value: None,
					offset: Offset(0)
				},
				Instruction::MovePtr(_),
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0)
				}
			] | [
				Instruction::MovePtr(_),
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0)
				},
				Instruction::MovePtr(_),
				Instruction::SetVal {
					value: None,
					offset: Offset(0)
				}
			] | [
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0)
				},
				Instruction::SetVal { value: None, .. }
			] | [
				Instruction::SetVal { value: None, .. },
				Instruction::IncVal {
					value: Value::Constant(-1),
					offset: Offset(0)
				}
			]
		)
	}
}
