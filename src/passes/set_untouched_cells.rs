use crate::{Change, Instruction, Pass};

// Currently only runs on the beginning cell, but can be expanded once cell analysis is introduced.
#[derive(Debug, Clone, Copy)]
pub struct SetUntouchedCellsPass;

impl Pass for SetUntouchedCellsPass {
	fn run_pass(&mut self, unit: &mut Vec<Instruction>) -> bool {
		if let Some(Instruction::Inc(i)) = unit.first() {
			Change::ReplaceOne(Instruction::Set(*i as u8)).apply(unit, 0, 1);

			true
		} else {
			false
		}
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
