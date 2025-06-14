use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantScaleValInstrPass;

impl PeepholePass for RemoveRedundantScaleValInstrPass {
	const SIZE: usize = 1;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::ScaleVal { factor: 0 }] => {
				Some(Change::replace(Instruction::clear_val()))
			}
			[Instruction::ScaleVal { factor: 1 }] => Some(Change::remove()),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [Instruction::ScaleVal { factor: 0 | 1 }])
	}
}
