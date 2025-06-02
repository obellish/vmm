use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeConstantSubPass;

impl PeepholePass for OptimizeConstantSubPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: Some(value),
					offset: None,
				},
				Instruction::SubCell { offset },
			] if i8::try_from(value.get()).is_ok() => {
				let value = i8::try_from(value.get()).ok()?;
				Some(Change::Replace(vec![
					Instruction::clear_val(),
					Instruction::inc_val_at(-value, *offset),
				]))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					value: Some(i),
					offset: None
				},
				Instruction::SubCell { .. }
			]
			if i8::try_from(i.get()).is_ok()
		)
	}
}
