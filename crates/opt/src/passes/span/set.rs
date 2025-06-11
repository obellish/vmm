use vmm_ir::{Instruction, SpanInstruction, SpanInstructionType};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSetSpanPass;

impl PeepholePass for OptimizeSetSpanPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: a,
					offset: x,
				},
				Instruction::SetVal {
					value: b,
					offset: y,
				},
			] if *a == *b => Some(Change::replace(Instruction::set_span(
				a.get_or_zero(),
				x.get_or_zero(),
				y.get_or_zero(),
			))),
			[
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Set { value: a },
					start,
					..
				}),
				Instruction::SetVal {
					value: b,
					offset: x,
				},
			]
			| [
				Instruction::SetVal {
					value: b,
					offset: x,
				},
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Set { value: a },
					start,
					..
				}),
			] if *a == *b => Some(Change::replace(Instruction::set_span(
				a.get_or_zero(),
				start,
				x.get_or_zero(),
			))),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		match window {
			[
				Instruction::SetVal {
					value: a,
					offset: x,
				},
				Instruction::SetVal {
					value: b,
					offset: y,
				},
			] if *a == *b => {
				let x = x.get_or_zero();
				let y = y.get_or_zero();

				x + 1 == y
			}
			[
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Set { value: a },
					end,
					..
				}),
				Instruction::SetVal {
					value: b,
					offset: x,
				},
			]
			| [
				Instruction::SetVal {
					value: b,
					offset: x,
				},
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Set { value: a },
					end,
					..
				}),
			] if *a == *b => {
				let x = x.get_or_zero();

				end + 1 == x
			}
			_ => false,
		}
	}
}
