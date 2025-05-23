use vmm_ir::Offset;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct SetUntouchedCellsPass {
	hit_pass: bool,
}

impl PeepholePass for SetUntouchedCellsPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if self.hit_pass {
			return None;
		}

		match window {
			[
				Instruction::FindZero(_)
				| Instruction::ScaleAndMoveVal { .. }
				| Instruction::DynamicLoop(_),
			] => {
				self.hit_pass = true;
				None
			}
			[Instruction::MovePtr(Offset::Relative(x))] if *x < 0 => {
				self.hit_pass = true;
				None
			}
			[
				Instruction::IncVal {
					value: x,
					offset: None,
				},
			] => {
				self.hit_pass = true;
				Some(Change::ReplaceOne(Instruction::set_val(*x as u8)))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		if self.hit_pass {
			return false;
		}

		matches!(
			window,
			[Instruction::FindZero(_)
				| Instruction::ScaleAndMoveVal { .. }
				| Instruction::DynamicLoop(_)
				| Instruction::MovePtr(Offset::Relative(isize::MIN..=0))
				| Instruction::IncVal { offset: None, .. }]
		)
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
