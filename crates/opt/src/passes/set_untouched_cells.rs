use crate::{Change, Instruction, Pass};

#[derive(Debug, Default)]
pub struct SetUntouchedCellsPass;

impl Pass for SetUntouchedCellsPass {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		if let Some(Instruction::IncVal(i)) = program.first() {
			Change::ReplaceOne(Instruction::SetVal(*i as u8)).apply(program, 0, 1);

			true
		} else {
			false
		}
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
