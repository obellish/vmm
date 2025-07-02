use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderSetIncPass;

impl PeepholePass for ReorderSetIncPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				b @ Instruction::SetVal { offset: y, .. },
				a @ Instruction::IncVal { offset: x, .. },
			] if *x != *y => Some(Change::swap([a.clone(), b.clone()])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: y, .. },
				Instruction::IncVal { offset: x, .. },
			]
			if *x != *y
		)
	}
}
