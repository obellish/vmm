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
					[Instruction::IncVal(-1), rest @ ..] | [rest @ .., Instruction::IncVal(-1)] => {
						let mut output = Vec::with_capacity((*i as usize) * rest.len());

						for _ in 0..*i {
							output.extend_from_slice(rest);
						}

						Some(Change::Replace(output))
					}
					_ => None,
				}
			}
			// [Instruction::IncVal(i), Instruction::RawLoop(inner)] if *i > 0 => {
			// 	println!("{i} {inner:?}");
			// 	if inner.iter().any(Instruction::has_side_effect) {
			// 		return None;
			// 	}

			// 	match inner.as_slice() {
			// 		[Instruction::IncVal(-1), rest @ ..] | [rest @ .., Instruction::IncVal(-1)] => {
			// 			let mut output =
			// 				Vec::with_capacity((*i as usize) * rest.len() + inner.len());

			// 			for _ in 0..*i {
			// 				output.extend_from_slice(rest);
			// 			}

			// 			output.push(Instruction::RawLoop(inner.clone()));

			// 			Some(Change::Replace(output))
			// 		}
			// 		_ => None,
			// 	}
			// }
			_ => None,
		}
	}
}
