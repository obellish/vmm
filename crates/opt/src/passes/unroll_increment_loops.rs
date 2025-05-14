use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollIncrementLoopsPass;

impl PeepholePass for UnrollIncrementLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::IncVal(i), Instruction::RawLoop(inner)] if *i > 0 => {
				if inner.iter().any(|i| i.has_side_effect() || i.is_loop()) {
					return None;
				}

				match inner.as_slice() {
					[Instruction::IncVal(-1), rest @ ..] | [rest @ .., Instruction::IncVal(-1)] => {
						let mut output =
							Vec::with_capacity((*i as u8 as usize) * rest.len() + inner.len());

						for _ in 0..(*i as u8) {
							output.extend_from_slice(rest);
						}

						output.push(Instruction::RawLoop(inner.clone()));

						Some(Change::Replace(output))
					}
					_ => None,
				}
			}
			_ => None,
		}
	}
}
