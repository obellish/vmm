use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveNoMovesOrChangePass;

impl PeepholePass for RemoveNoMovesOrChangePass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::MovePtr(Offset::Relative(0))] => Some(Change::Remove),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::MovePtr(Offset::Relative(0)) | Instruction::IncVal { value: 0, .. }]
		)
	}
}
