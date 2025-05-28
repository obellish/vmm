use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct UnrollOneOffLoopsPass;

impl LoopPass for UnrollOneOffLoopsPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				rest @ ..,
				i @ (Instruction::SetVal {
					offset: None,
					value: None,
				}
				| Instruction::FindZero(..)
				| Instruction::ScaleAndMoveVal { .. }),
			] if !rest.iter().any(|i| i.is_loop() || i.has_side_effect()) => {
				let mut out = rest.to_vec();

				out.push(i.clone());

				Some(Change::Replace(out))
			}
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		let [
			rest @ ..,
			Instruction::FindZero(..)
			| Instruction::SetVal {
				offset: None,
				value: None,
			}
			| Instruction::ScaleAndMoveVal { .. },
		] = loop_values
		else {
			return false;
		};

		if rest.iter().any(|i| i.is_loop() || i.has_side_effect()) {
			return false;
		}

		true
	}
}
