use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default, Clone, Copy)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::Set(x), Instruction::Loop(instructions)] = window {
			let mut output = Vec::new();

			let mut instructions = instructions.clone();

			let mut removed_instruction = false;

			let first_instr = instructions.first()?;

			if matches!(first_instr, Instruction::Add(x) if *x < 0) {
				instructions.remove(0);
				removed_instruction = true;
			}

			if !removed_instruction {
				let last_instr = instructions.last()?;

				if matches!(last_instr, Instruction::Add(x) if *x < 0) {
					instructions.pop();
					removed_instruction = true;
				}
			}

			if !removed_instruction {
				return None;
			}

			for i in 0..*x {
				output.extend_from_slice(&instructions);
			}

			Some(Change::Replace(output))
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("unroll constant loops")
	}

	fn should_run_on_loop(&self) -> bool {
		false // just a precaution
	}
}
