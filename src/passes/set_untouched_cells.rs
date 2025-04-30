use std::borrow::Cow;

use tracing::trace;

use crate::{Change, ExecutionUnit, Instruction, Pass};

// Currently only runs on the beginning cell, but can be expanded once cell analysis is introduced.
#[derive(Debug)]
pub struct SetUntouchedCells;

impl Pass for SetUntouchedCells {
	fn run_pass(&self, unit: &mut ExecutionUnit) -> bool {
		if let Some(Instruction::Add(i)) = unit.program().first() {
			unit.program_mut().as_raw()[0] = Instruction::Set(*i as u8);

			true
		} else {
			false
		}
	}

	fn name(&self) -> std::borrow::Cow<'static, str> {
		Cow::Borrowed("set untouched cells")
	}
}
