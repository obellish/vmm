use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantCompilerHintsPass;

impl PeepholePass for RemoveRedundantCompilerHintsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::MoveVal(..), Instruction::Hint(..)] => Some(Change::remove_offset(1)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::MoveVal(..), Instruction::Hint(..)])
	}
}
