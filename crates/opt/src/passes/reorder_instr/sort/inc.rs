use vmm_ir::Instruction;
use vmm_utils::Sorted;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct SortIncInstrPass;

impl PeepholePass for SortIncInstrPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			window
				if window.iter().all(Instruction::is_inc_val)
					&& !window.is_sorted_by_key(Instruction::offset) =>
			{
				Some(Change::Replace(
					window.to_owned().sorted_by_key(Instruction::offset),
				))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		window.iter().all(Instruction::is_inc_val) && !window.is_sorted_by_key(Instruction::offset)
	}
}
