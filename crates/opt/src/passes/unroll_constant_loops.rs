use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::SetVal(i), Instruction::RawLoop(inner)] => {
				if inner.iter().any(Instruction::has_side_effect) {
					return None;
				}
				match inner.as_slice() {
					[
						Instruction::IncVal {
							value: -1,
							offset: None,
						},
						rest @ ..,
					]
					| [
						rest @ ..,
						Instruction::IncVal {
							value: -1,
							offset: None,
						},
					] => {
						let mut output = Vec::with_capacity((*i as usize) * rest.len());

						for _ in 0..*i {
							output.extend_from_slice(rest);
						}

						Some(Change::Replace(output))
					}
					_ => None,
				}
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [Instruction::SetVal(_), Instruction::RawLoop(inner)] = window else {
			return false;
		};

		if inner.iter().any(Instruction::has_side_effect) {
			return false;
		}

		true
	}
}
