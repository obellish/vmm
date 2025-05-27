use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeFindZeroThenIncValPass;

impl PeepholePass for OptimizeFindZeroThenIncValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::FindZero(i),
				Instruction::IncVal {
					value,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::find_zero(*i),
				Instruction::set_val(*value as u8),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::FindZero(_),
				Instruction::IncVal { offset: None, .. }
			]
		)
	}
}
