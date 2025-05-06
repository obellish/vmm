use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Set(i), Instruction::Loop(inner)] => {
				if inner.iter().any(|i| matches!(i, Instruction::Loop(_))) {
					return None;
				}
				let mut inner = inner.clone();

				let mut decrement_removed = false;

				let first_instr = inner.first()?;

				if matches!(first_instr, Instruction::Add(x) if *x < 0) {
					inner.remove(0);
					decrement_removed = true;
				}

				if !decrement_removed {
					let last_instr = inner.last()?;

					if matches!(last_instr, Instruction::Add(x) if *x < 0) {
						inner.pop();
						decrement_removed = true;
					}
				}

				if !decrement_removed {
					return None;
				}

				println!("{inner:?}");

				None
			}
			_ => None,
		}
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
