use std::borrow::Cow;

use tracing::trace;

use crate::{Change, ExecutionUnit, Instruction, Pass};

// Currently only runs on the beginning cell, but can be expanded once cell analysis is introduced.
#[derive(Debug)]
pub struct SetUntouchedCells;

impl Pass for SetUntouchedCells {
	fn run_pass(&mut self, unit: &mut ExecutionUnit) -> bool {
		if matches!(unit.program().first(), Some(&Instruction::Add(_))) {
			let instr = unit.program_mut().as_raw().remove(0);

			let Instruction::Add(value) = instr else {
				panic!("checked for add, got something else");
			};

			let value = value as u8;

			trace!("setting cell 0 to {value}");

			*unit.tape_mut().current_cell_mut() = value;
		}

		false
	}

	fn name(&self) -> std::borrow::Cow<'static, str> {
		Cow::Borrowed("set untouched cells")
	}
}
