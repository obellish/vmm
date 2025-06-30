use std::num::NonZeroU8;

use vmm_ir::{Instruction, Offset};
use vmm_iter::IteratorExt as _;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct SortSetInstrPass;

impl PeepholePass for SortSetInstrPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		Some(Change::swap(
			window.iter().cloned().sorted_by_key(sorter_key),
		))
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		window.iter().all(Instruction::is_set_val) && !window.iter().is_sorted_by_key(sorter_key)
	}
}

fn sorter_key(a: &Instruction) -> (Offset, Option<NonZeroU8>) {
	(a.offset().get_or_zero(), get_set_value(a))
}

const fn get_set_value(i: &Instruction) -> Option<NonZeroU8> {
	match i {
		Instruction::SetVal { value, .. } => *value,
		_ => None,
	}
}
