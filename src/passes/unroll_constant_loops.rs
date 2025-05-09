use crate::{Change, Instruction, PeepholePass, StackedInstruction};

#[derive(Debug, Default, Clone, Copy)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::SetVal(i), Instruction::RawLoop(inner)] => {
				if inner.iter().any(Instruction::is_loop) {
					return None;
				}

				let mut inner = inner.clone();

				let mut decrement_removed = false;

				let first_instr = inner.first()?;

				if matches!(
					first_instr,
					Instruction::Stacked(StackedInstruction::IncVal(-1))
				) {
					inner.remove(0);
					decrement_removed = true;
				}

				if !decrement_removed {
					let last_instr = inner.last()?;

					if matches!(
						last_instr,
						Instruction::Stacked(StackedInstruction::IncVal(-1))
					) {
						inner.pop();
						decrement_removed = true;
					}
				}

				if !decrement_removed {
					return None;
				}

				let mut output = Vec::with_capacity((*i as usize) * inner.len());

				for _ in 0..*i {
					output.extend_from_slice(&inner);
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
