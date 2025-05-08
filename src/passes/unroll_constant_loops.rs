use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::SetVal(i), Instruction::RawLoop(inner)] => {
				if inner.iter().any(|i| matches!(i, Instruction::RawLoop(_))) {
					return None;
				}
				let mut inner = inner.clone();

				let mut decrement_removed = false;

				let first_instr = inner.first()?;

				if matches!(first_instr, Instruction::IncVal(x) if *x == -1) {
					inner.remove(0);
					decrement_removed = true;
				}

				if !decrement_removed {
					let last_instr = inner.last()?;

					if matches!(last_instr, Instruction::IncVal(x) if *x == -1) {
						inner.pop();
						decrement_removed = true;
					}
				}

				if !decrement_removed {
					return None;
				}

				let mut output = Vec::new();

				for _ in 0..*i {
					output.extend(inner.clone());
				}

				Some(Change::Replace(output))
			}
			_ => None,
		}
	}

	fn should_run_on_loop(&self) -> bool {
		true
	}
}
