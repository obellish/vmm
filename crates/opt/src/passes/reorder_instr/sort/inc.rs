use std::cmp::Ordering;

use vmm_ir::{Instruction, Offset, SimdInstruction};
use vmm_utils::Sorted;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct SortIncInstrPass;

impl PeepholePass for SortIncInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		// match window {
		// 	window
		// 		if window.iter().all(Instruction::is_inc_val)
		// 			&& !window.is_sorted_by_key(offsets_and_values) =>
		// 	{
		// 		Some(Change::Replace(
		// 			window.to_owned().sorted_by_key(offsets_and_values),
		// 		))
		// 	}
		// 	_ => None,
		// }
		Some(Change::Replace(window.to_owned().sorted_by(sorter)))
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		window.iter().all(Instruction::is_inc_val) && {
			let cloned = window.to_owned();

			cloned.sorted_by(sorter) != window
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
