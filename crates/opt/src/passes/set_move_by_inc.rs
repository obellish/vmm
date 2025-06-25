use vmm_ir::{Instruction, Offset, Value};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSetIncByCellPass;

impl PeepholePass for OptimizeSetIncByCellPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					offset: Offset(0),
					value,
				},
				Instruction::IncVal {
					value: Value::CellAt(Offset(0)),
					offset,
				},
			] => Some(Change::swap([
				Instruction::set_val(value.get_or_zero()),
				Instruction::set_val_at(value.get_or_zero(), offset),
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
				Instruction::IncVal {
					value: Value::CellAt(Offset(0)),
					..
				}
			]
		)
	}
}
