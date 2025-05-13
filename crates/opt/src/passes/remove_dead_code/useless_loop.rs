use crate::{Change, Instruction, Pass};

#[derive(Debug, Default)]
pub struct RemoveUselessLoopsPass;

impl Pass for RemoveUselessLoopsPass {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		if let Some(Instruction::RawLoop(_)) = program.first() {
			Change::Remove.apply(program, 0, 1);

			true
		} else {
			false
		}
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
