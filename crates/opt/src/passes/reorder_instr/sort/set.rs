use std::{cmp::Ordering, num::NonZeroU8};

use itertools::Itertools as _;
use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct SortSetInstrPass;

impl PeepholePass for SortSetInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		Some(Change::swap(window.iter().cloned().sorted_by(sorter)))
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		window.iter().all(Instruction::is_set_val) && {
			let cloned = window.to_owned();

			cloned.into_iter().sorted_by(sorter).collect_vec() != window
		}
	}
}

fn sorter(a: &Instruction, b: &Instruction) -> Ordering {
	a.offset()
		.cmp(&b.offset())
		.then(get_set_value(a).cmp(&get_set_value(b)))
}

const fn get_set_value(i: &Instruction) -> Option<NonZeroU8> {
	match i {
		Instruction::SetVal { value, .. } => *value,
		_ => None,
	}
}
