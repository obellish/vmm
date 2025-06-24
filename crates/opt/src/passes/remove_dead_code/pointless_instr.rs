use vmm_ir::{Instruction, Offset, Value};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemovePointlessInstrPass;

impl PeepholePass for RemovePointlessInstrPass {
	const SIZE: usize = 1;

	#[inline]
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::remove())
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::MovePtr(Offset(0))
				| Instruction::IncVal {
					value: Value::Constant(0),
					..
				}]
		)
	}
}
