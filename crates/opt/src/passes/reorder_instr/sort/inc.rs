use itertools::Itertools as _;
use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct SortIncInstrPass;

impl PeepholePass for SortIncInstrPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		Some(Change::swap(
			window.iter().cloned().sorted_by_key(sorter_key),
		))
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		window.iter().all(Instruction::is_change_val) && !window.iter().is_sorted_by_key(sorter_key)
	}
}

fn sorter_key(instr: &Instruction) -> (Offset, Option<i8>) {
	(instr.offset().get_or_zero(), get_inc_value(instr))
}

const fn get_inc_value(i: &Instruction) -> Option<i8> {
	match i {
		Instruction::IncVal { value, .. } => Some(*value),
		_ => None,
	}
}
