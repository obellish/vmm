use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantScaleValInstrPass;

impl PeepholePass for RemoveRedundantScaleValInstrPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::ReplaceOne(Instruction::clear_val()))
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::ScaleVal { factor: 0 }])
	}
}
