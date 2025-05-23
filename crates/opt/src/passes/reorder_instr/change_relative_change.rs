use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderChangeChangeRelativeChangeInstrsPass;

impl PeepholePass for ReorderChangeChangeRelativeChangeInstrsPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					offset: None,
					value: a,
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(x)),
					value: b,
				},
				Instruction::IncVal {
					offset: None,
					value: c,
				},
			] => Some(Change::Replace(vec![
				Instruction::inc_val(*a),
				Instruction::inc_val(*c),
				Instruction::inc_val_at(*b, x),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::IncVal {
					offset: Some(Offset::Relative(_)),
					..
				},
				Instruction::IncVal { offset: None, .. }
			]
		)
	}
}
