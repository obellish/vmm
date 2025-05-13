use crate::{Change, Instruction, Pass};

#[derive(Debug, Default)]
pub struct SetUntouchedCellsPass {
	hit_pass: bool,
}

impl Pass for SetUntouchedCellsPass {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		// if let Some(Instruction::IncVal(i)) = program.first() {
		// 	Change::ReplaceOne(Instruction::SetVal(*i as u8)).apply(program, 0, 1);

		// 	true
		// } else {
		// 	false
		// }

		let mut progress = false;

		if self.hit_pass {
			return progress;
		}

		let mut changes = Vec::new();

		for (idx, instr) in program.iter_mut().enumerate() {
			match instr {
				Instruction::FindZero(_)
				| Instruction::MoveVal { .. }
				| Instruction::RawLoop(_) => {
					self.hit_pass = true;
					break;
				}
				Instruction::MovePtr(i) if *i < 0 => {
					self.hit_pass = true;
					break;
				}
				Instruction::IncVal(i) => {
					// *instr = Instruction::SetVal(*i as u8);
					// Change::ReplaceOne(Instruction::SetVal(*i as u8)).apply(program, idx, 1);
					changes.push((Change::ReplaceOne(Instruction::SetVal(*i as u8)), idx, 1));
				}
				_ => {}
			}
		}

		for (change, idx, size) in changes {
			change.apply(program, idx, size);
			progress = true;
		}

		progress
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
