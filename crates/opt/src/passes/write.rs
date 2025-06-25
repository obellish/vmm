use vmm_ir::{Instruction, Offset, Value};
use vmm_num::ops::WrappingAdd;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeWritePass;

impl PeepholePass for OptimizeWritePass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value,
					offset: Offset(0),
				},
				Instruction::Write {
					value: Value::CellAt(Offset(0)),
				},
			] => Some(Change::replace(Instruction::write_byte(
				value.get_or_zero(),
			))),
			[
				Instruction::Write {
					value: Value::Constant(b),
				},
				Instruction::IncVal {
					value: Value::Constant(value),
					offset: Offset(0),
				},
			] => {
				let last = b.last().copied()?;

				Some(Change::swap([
					Instruction::write_value(Value::Constant(b.clone())),
					Instruction::set_val(WrappingAdd::wrapping_add(last, value)),
				]))
			}
			[
				Instruction::Write {
					value: Value::Constant(a),
				},
				Instruction::Write {
					value: Value::Constant(b),
				},
			] => Some(Change::replace(Instruction::write_value(Value::Constant(
				a.clone() + b.clone(),
			)))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Write {
					value: Value::Constant(..)
				},
				Instruction::Write {
					value: Value::Constant(..)
				} | Instruction::IncVal {
					offset: Offset(0),
					value: Value::Constant(..)
				}
			] | [
				Instruction::SetVal {
					offset: Offset(0),
					..
				},
				Instruction::Write {
					value: Value::CellAt(Offset(0))
				}
			]
		)
	}
}
