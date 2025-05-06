use std::borrow::Cow;

use tracing::trace;

use crate::{Change, ExecutionUnit, Instruction, Pass, Program};

// Currently only runs on the beginning cell, but can be expanded once cell analysis is introduced.
#[derive(Debug, Clone, Copy)]
pub struct SetUntouchedCells;

impl Pass for SetUntouchedCells {
	fn run_pass(&self, unit: &mut Vec<Instruction>) -> bool {
		if let Some(Instruction::Add(i)) = unit.first() {
			Change::ReplaceOne(Instruction::Set(*i as u8)).apply(unit, 0, 1);

			true
		} else {
			false
		}
	}

	fn name(&self) -> std::borrow::Cow<'static, str> {
		Cow::Borrowed("set untouched cells")
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
