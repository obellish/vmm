use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemovePointlessInstrPass;

impl PeepholePass for RemovePointlessInstrPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		Some(Change::remove())
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::MovePtr(Offset(0))
				| Instruction::IncVal { value: 0, .. }
				| Instruction::Write { count: 0, .. }]
		)
	}
}
