use vmm_ir::{BlockInstruction, HasIo as _, Instruction, Offset, PtrMovement as _};
use vmm_span::Span;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollConstantLoopsPass;

impl PeepholePass for UnrollConstantLoopsPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					offset: Offset(0),
					value: Some(i),
				},
				Instruction::Block(BlockInstruction::DynamicLoop(inner)),
			] => {
				if inner.iter().any(Instruction::has_io) {
					return None;
				}

				if !matches!(inner.ptr_movement(), Some(Offset(0))) {
					return None;
				}

				match &**inner {
					[
						Instruction::IncVal {
							value,
							offset: Offset(0),
						},
						rest @ ..,
					]
					| [
						rest @ ..,
						Instruction::IncVal {
							value,
							offset: Offset(0),
						},
					] if *value < 0 && matches!(i.get() % value.unsigned_abs(), 0) => {
						let mut output = Vec::with_capacity((i.get() as usize) * rest.len());

						Span::from(0..i.get())
							.into_iter()
							.step_by(value.unsigned_abs() as usize)
							.for_each(|_| output.extend_from_slice(rest));

						Some(Change::swap(output))
					}
					_ => None,
				}
			}
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		let [
			Instruction::SetVal {
				offset: Offset(0),
				value: Some(i),
			},
			Instruction::Block(BlockInstruction::DynamicLoop(inner)),
		] = window
		else {
			return false;
		};

		if inner.iter().any(Instruction::has_io) {
			return false;
		}

		if !matches!(inner.ptr_movement(), Some(Offset(0))) {
			return false;
		}

		matches!(
			&**inner,
			[
				Instruction::IncVal {
					offset: Offset(0),
					value
				},
				..
			] | [
				..,
				Instruction::IncVal {
					offset: Offset(0),
					value
				}
			]
			if *value < 0 && matches!(i.get() % value.unsigned_abs(), 0)
		)
	}
}
