use vmm_ir::{CompilerHint, Instruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantCompilerHintsPass;

impl PeepholePass for RemoveRedundantCompilerHintsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::remove_offset(1))
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[i, Instruction::Hint(CompilerHint::KnownValue { .. })]
			if !matches!(i, Instruction::Block(..))
		)
	}
}
