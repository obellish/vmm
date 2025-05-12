use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::SetVal(i), Instruction::RawLoop(inner)] => {
				if inner.iter().any(Instruction::is_loop) {
					return None;
				}
				match inner.as_slice() {
					[Instruction::IncVal(-1), rest @ ..] | [rest @ .., Instruction::IncVal(-1)] => {
						// Some(Change::ReplaceOne(Instruction::ConstantLoop(
						// 	*i,
						// 	rest.to_owned(),
						// )))
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
}
