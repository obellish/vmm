use crate::{Change, Instruction, PeepholePass};

const MAX_LOOP_UNROLLING: usize = 5;

#[derive(Debug, Default)]
pub struct UnrollIncrementLoopsPass;

impl PeepholePass for UnrollIncrementLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: i,
					offset: None,
				},
				raw_loop @ Instruction::DynamicLoop(inner),
			] if *i > 0
				&& !raw_loop.might_move_ptr()
				&& (raw_loop.nested_loops() < MAX_LOOP_UNROLLING) =>
			{
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
						let mut output =
							Vec::with_capacity((*i as u8 as usize) * rest.len() + inner.len());

						for _ in 0..(*i as u8) {
							output.extend_from_slice(rest);
						}

						output.push(Instruction::DynamicLoop(inner.clone()));

						Some(Change::Replace(output))
					}
					_ => None,
				}
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [
			Instruction::IncVal {
				value: i,
				offset: None,
			},
			raw_loop @ Instruction::DynamicLoop(inner),
		] = window
		else {
			return false;
		};

		if *i <= 0 {
			return false;
		}

		if raw_loop.might_move_ptr() {
			return false;
		}

		if raw_loop.nested_loops() >= MAX_LOOP_UNROLLING {
			return false;
		}

		matches!(
			inner.as_slice(),
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				..,
			] | [
				..,
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			]
		)
	}
}
