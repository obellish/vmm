use vmm_ir::{Instruction, Offset, SpanInstructionType};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseRelativeInstrPass;

impl PeepholePass for CollapseRelativeInstrPass {
	const SIZE: usize = 3;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value,
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
			] if *x == -y => Some(Change::replace(Instruction::inc_val_at(*value, x))),
			[
				Instruction::MovePtr(x),
				Instruction::SetVal {
					value,
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
			] if *x == -y => Some(Change::replace(Instruction::set_val_at(
				value.get_or_zero(),
				*x,
			))),
			[
				Instruction::MovePtr(x),
				Instruction::Write {
					offset: Offset(0),
					count,
				},
				Instruction::MovePtr(y),
			] if *x == -y => Some(Change::replace(Instruction::write_many_at(*count, *x))),
			[
				Instruction::MovePtr(x),
				Instruction::Span(span),
				Instruction::MovePtr(y),
			] if *x == -y => Some(Change::replace(match span.ty() {
				SpanInstructionType::Inc { value } => {
					Instruction::inc_span(value, span.start + x, span.end + x)
				}
				SpanInstructionType::Set { value } => {
					Instruction::set_span(value.get_or_zero(), span.start + x, span.end + x)
				}
				_ => return None,
			})),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::MovePtr(x),
				Instruction::IncVal { offset: Offset(0), .. }
					| Instruction::SetVal { offset: Offset(0), .. }
					| Instruction::Write { offset: Offset(0), .. }
					| Instruction::Span(..),
				Instruction::MovePtr(y)
			]
			if *x == -y
		)
	}
}
