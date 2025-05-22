use crate::{Change, Instruction, Pass};

#[derive(Debug, Default)]
pub struct CleanUpStartPass;

impl Pass for CleanUpStartPass {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		let Some(instr) = program.first().cloned() else {
			return false;
		};

		match instr {
			Instruction::RawLoop(_)
			| Instruction::SetVal {
				value: None,
				offset: None,
			} => {
				Change::Remove.apply(program, 0, 1);
				true
			}
			_ => false,
		}
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
