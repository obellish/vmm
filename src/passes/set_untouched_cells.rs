use std::borrow::Cow;

use crate::{Change, ExecutionUnit, Instruction, Pass};

#[derive(Debug)]
pub struct SetUntouchedCells;

impl Pass for SetUntouchedCells {
	fn run_pass(&self, unit: &mut ExecutionUnit) -> bool {
        if matches!(unit.program().first(), Some(&Instruction::Add(_))) {
            let instr = unit.program_mut().as_raw().remove(0);

            let Instruction::Add(value) = instr else {
                panic!("checked for add, got something else");
            };

            *unit.tape_mut().current_cell_mut() = value as u8;
        }

        false
    }

	fn name(&self) -> std::borrow::Cow<'static, str> {
		Cow::Borrowed("set untouched cells")
	}
}
