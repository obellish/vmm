use vmm_ir::{BlockInstruction, Instruction, Offset, PtrMovement};
use vmm_span::Span;

use crate::{Change, PeepholePass};

const MAX_LOOP_UNROLLING: usize = 5;

#[derive(Debug, Default)]
pub struct UnrollIncrementLoopsPass;

impl PeepholePass for UnrollIncrementLoopsPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: i,
					offset: Offset(0),
				},
				raw_loop @ Instruction::Block(BlockInstruction::DynamicLoop(inner)),
			] if *i > 0
				&& !raw_loop.might_move_ptr()
				&& (raw_loop.nested_loops() < MAX_LOOP_UNROLLING) =>
			{
				match &**inner {
					[
						Instruction::IncVal {
							value: -1,
							offset: Offset(0),
						},
						rest @ ..,
					]
					| [
						rest @ ..,
						Instruction::IncVal {
							value: -1,
							offset: Offset(0),
						},
					] => {
						let mut output =
							Vec::with_capacity((*i as u8 as usize) * rest.len() + inner.len());

						Span::from(0..(*i as u8))
							.into_iter()
							.for_each(|_| output.extend_from_slice(rest));

						output.push(Instruction::Block(BlockInstruction::DynamicLoop(
							inner.clone(),
						)));

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
			Instruction::IncVal {
				value: i,
				offset: Offset(0),
			},
			raw_loop @ Instruction::Block(BlockInstruction::DynamicLoop(inner)),
		] = window
		else {
			return false;
		};

		if *i <= 0 {
			return false;
		}

		if !matches!(inner.ptr_movement(), Some(0)) {
			return false;
		}

		if raw_loop.nested_loops() >= MAX_LOOP_UNROLLING {
			return false;
		}

		matches!(
			&**inner,
			[
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
				},
				..,
			] | [
				..,
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
				},
			]
		)
	}
}
