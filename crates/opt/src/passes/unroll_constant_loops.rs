use vmm_ir::{BlockInstruction, Instruction, PtrMovement};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					offset: None,
					value: Some(i),
				},
				Instruction::Block(BlockInstruction::DynamicLoop(inner)),
			] => {
				if inner.iter().any(Instruction::has_io) {
					return None;
				}

				if !matches!(inner.ptr_movement(), Some(0)) {
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
						let mut output = Vec::with_capacity((i.get() as usize) * rest.len());

						for _ in 0..i.get() {
							output.extend_from_slice(rest);
						}

						Some(Change::Swap(output))
					}
					_ => None,
				}
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [
			Instruction::SetVal { offset: None, .. },
			Instruction::Block(BlockInstruction::DynamicLoop(inner)),
		] = window
		else {
			return false;
		};

		if inner.iter().any(Instruction::has_io) {
			return false;
		}

		if !matches!(inner.ptr_movement(), Some(0)) {
			return false;
		}

		matches!(
			inner.as_slice(),
			[
				Instruction::IncVal {
					offset: None,
					value: -1
				},
				..
			] | [
				..,
				Instruction::IncVal {
					offset: None,
					value: -1
				}
			]
		)
	}
}
