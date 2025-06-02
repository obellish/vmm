use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantScaleValInstrPass;

impl PeepholePass for RemoveRedundantScaleValInstrPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::ScaleVal { factor: 0 }] => {
				Some(Change::Replace(Instruction::clear_val()))
			}
			[Instruction::ScaleVal { factor: 1 }] => Some(Change::Remove),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::ScaleVal { factor: 0 | 1 }])
	}
}
