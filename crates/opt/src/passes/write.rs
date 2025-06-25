use vmm_ir::{Instruction, Offset, Value};
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
			] => Some(Change::swap([
				Instruction::set_val(value.get_or_zero()),
				Instruction::write_byte(value.get_or_zero()),
			])),

			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
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
