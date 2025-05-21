use vmm_ir::MoveBy;

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
			[Instruction::FindZero(_) | Instruction::MoveVal { .. } | Instruction::RawLoop(_)] => {
				self.hit_pass = true;
				None
			}
			[Instruction::MovePtr(MoveBy::Relative(x))] if *x < 0 => {
				self.hit_pass = true;
				None
			}
			[Instruction::IncVal(x, None)] => {
				self.hit_pass = true;
				Some(Change::ReplaceOne(Instruction::SetVal(*x as u8)))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		!self.hit_pass
			&& matches!(
				window,
				[Instruction::FindZero(_)
					| Instruction::MoveVal { .. }
					| Instruction::RawLoop(_)
					| Instruction::MovePtr(MoveBy::Relative(isize::MIN..=0))
					| Instruction::IncVal(_, None)]
			)
	}

	fn should_run_on_loop(&self) -> bool {
		false
	}
}
