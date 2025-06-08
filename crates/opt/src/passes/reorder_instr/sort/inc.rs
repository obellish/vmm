use std::cmp::Ordering;

use itertools::Itertools as _;
use vmm_ir::{Instruction, SpanInstruction, SpanInstructionType};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct SortIncInstrPass;

impl PeepholePass for SortIncInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		Some(Change::swap(window.iter().cloned().sorted_by(sorter)))
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		window.iter().all(Instruction::is_change_val) && {
			let cloned = window.to_owned();

			cloned.into_iter().sorted_by(sorter).collect_vec() != window
		}
	}
}

fn sorter(a: &Instruction, b: &Instruction) -> Ordering {
	a.offset()
		.get_or_zero()
		.cmp(&b.offset().get_or_zero())
		.then(get_inc_value(a).cmp(&get_inc_value(b)))
}

const fn get_inc_value(i: &Instruction) -> Option<i8> {
	match i {
		Instruction::IncVal { value, .. }
		| Instruction::Span(SpanInstruction {
			ty: SpanInstructionType::Inc { value },
			..
		}) => Some(*value),
		_ => None,
	}
}
