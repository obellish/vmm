use std::cmp::Ordering;

use itertools::Itertools as _;
use vmm_ir::{Instruction, SimdInstruction};

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
		.cmp(&b.offset())
		.then(get_inc_value(a).cmp(&get_inc_value(b)))
}

const fn get_inc_value(i: &Instruction) -> Option<i8> {
	match i {
		Instruction::Simd(SimdInstruction::IncVals { value, .. })
		| Instruction::IncVal { value, .. } => Some(*value),
		_ => None,
	}
}
