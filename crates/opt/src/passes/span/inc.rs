use vmm_ir::{Instruction, SpanInstruction, SpanInstructionType};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeIncSpanPass;

impl PeepholePass for OptimizeIncSpanPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: 1,
					offset: x,
				},
				Instruction::IncVal {
					value: 1,
					offset: y,
				},
			] => Some(Change::replace(Instruction::inc_span(
				1,
				x.get_or_zero(),
				y.get_or_zero(),
			))),
			[
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Inc { value: a },
					start,
					..
				}),
				Instruction::IncVal {
					value: b,
					offset: x,
				},
			] if *a == *b => Some(Change::replace(Instruction::inc_span(
				*a,
				start,
				x.get_or_zero(),
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		match window {
			[
				Instruction::IncVal {
					value: 1,
					offset: x,
				},
				Instruction::IncVal {
					value: 1,
					offset: y,
				},
			] => {
				let x = x.get_or_zero();
				let y = y.get_or_zero();

				x + 1 == y
			}
			[
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Inc { value: a },
					end,
					..
				}),
				Instruction::IncVal {
					value: b,
					offset: x,
				},
			] if *a == *b => {
				let x = x.get_or_zero();

				end + 1 == x
			}
			_ => false,
		}
	}
}
