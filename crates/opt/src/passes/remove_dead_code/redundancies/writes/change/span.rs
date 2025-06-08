use vmm_ir::{Instruction, SpanInstruction, SpanInstructionType};
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantChangeValSpanPass;

impl PeepholePass for RemoveRedundantChangeValSpanPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Span(a), Instruction::Span(b)] if *a == *b => {
				Some(Change::remove_offset(1))
			}
			[
				Instruction::Span(
					span @ SpanInstruction {
						ty: SpanInstructionType::Set { value: a },
						..
					},
				),
				Instruction::SetVal {
					value: b,
					offset: x,
				},
			] if span.span().contains(&x.get_or_zero()) && a == b => Some(Change::remove_offset(1)),
			[
				Instruction::SetVal {
					value: b,
					offset: x,
				},
				Instruction::Span(
					span @ SpanInstruction {
						ty: SpanInstructionType::Set { value: a },
						..
					},
				),
			] if span.span().contains(&x.get_or_zero()) && a == b => Some(Change::remove_offset(0)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		match window {
			[Instruction::Span(a), Instruction::Span(b)] if *a == *b => true,
			[
				Instruction::Span(
					span @ SpanInstruction {
						ty: SpanInstructionType::Set { value: a },
						..
					},
				),
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
				Instruction::Span(
					span @ SpanInstruction {
						ty: SpanInstructionType::Set { value: a },
						..
					},
				),
			] if span.span().contains(&x.get_or_zero()) && a == b => true,
			_ => false,
		}
	}
}
