use vmm_ir::{Instruction, SpanInstruction, SpanInstructionType};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeDupeValPass;

impl LoopPass for OptimizeDupeValPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::Span(
					span @ SpanInstruction {
						ty: SpanInstructionType::Inc { value: 1 },
						..
					},
				),
			] => Some(Change::replace(Instruction::dupe_val(
				span.span().into_iter().collect(),
			))),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(2))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::Span(SpanInstruction {
					ty: SpanInstructionType::Inc { value: 1 },
					..
				})
			]
		)
	}
}
