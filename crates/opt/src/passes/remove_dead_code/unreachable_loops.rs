use vmm_ir::{Instruction, IsZeroingCell as _};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnreachableLoopsPass;

impl PeepholePass for RemoveUnreachableLoopsPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[i, Instruction::Block(..)] if i.is_zeroing_cell() => Some(Change::remove_offset(1)),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [i, Instruction::Block(..)] if i.is_zeroing_cell())
	}
}
