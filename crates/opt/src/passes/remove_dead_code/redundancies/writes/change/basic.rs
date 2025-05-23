use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantChangeValBasicPass;

impl PeepholePass for RemoveRedundantChangeValBasicPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::SetVal {
					offset: None,
					value: None,
				},
			] => Some(Change::ReplaceOne(Instruction::clear_val())),
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::SetVal {
					value: Some(x),
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::set_val(x.get()))),
			[
				Instruction::SetVal {
					value: None,
					offset: None,
				},
				Instruction::IncVal {
					value: y,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::set_val(*y as u8))),
			[
				Instruction::SetVal {
					offset: None,
					value: Some(x),
				},
				Instruction::IncVal {
					value: y,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::set_val(
				(x.get() as i8).wrapping_add(*y) as u8,
			))),
			[Instruction::IncVal { offset: None, .. }, Instruction::Read] => {
				Some(Change::ReplaceOne(Instruction::Read))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::SetVal { offset: None, .. } | Instruction::Read
			] | [
				Instruction::SetVal { offset: None, .. },
				Instruction::IncVal { offset: None, .. } | Instruction::Read
			]
		)
	}
}
