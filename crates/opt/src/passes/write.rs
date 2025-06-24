use vmm_ir::{Instruction, Value};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeWritePass;

impl PeepholePass for OptimizeWritePass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal { value, offset: x },
				Instruction::Write {
					value: Value::CellAt(y),
					count,
				},
			] if *x == *y => Some(Change::swap([
				Instruction::move_ptr(x),
				Instruction::write_value(*count, Value::Constant(value.get_or_zero())),
				Instruction::set_val(value.get_or_zero()),
				Instruction::move_ptr(-x),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: x, .. },
				Instruction::Write {
					value: Value::CellAt(y),
					..
				}
			]
			if *x == *y
		)
	}
}
