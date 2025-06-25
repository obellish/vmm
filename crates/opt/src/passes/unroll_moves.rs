use vmm_ir::{Instruction, Offset, Value};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollMovePass;

impl PeepholePass for UnrollMovePass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::MoveVal(offset)] => Some(Change::swap([
				Instruction::IncVal {
					value: Value::CellAt(Offset(0)),
					offset: *offset,
				},
				Instruction::clear_val(),
			])),
			[Instruction::TakeVal(offset)] => Some(Change::swap([
				Instruction::IncVal {
					value: Value::CellAt(Offset(0)),
					offset: *offset,
				},
				Instruction::clear_val(),
				Instruction::move_ptr(offset),
			])),
			[Instruction::FetchVal(offset)] => Some(Change::swap([
				Instruction::IncVal {
					offset: Offset(0),
					value: Value::CellAt(*offset),
				},
				Instruction::clear_val_at(*offset),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::MoveVal(..) | Instruction::TakeVal(..) | Instruction::FetchVal(..)]
		)
	}
}
