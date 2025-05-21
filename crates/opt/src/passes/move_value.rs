use vmm_ir::Offset;

use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct MoveValuePass;

impl LoopPass for MoveValuePass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal {
					value: j,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y)),
			]
			| [
				Instruction::IncVal {
					value: j,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y)),
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(x)),
			] if *x == -y => {
				let j = *j;
				let x = *x;

				if j < 0 {
					return None;
				}

				Some(Change::ReplaceOne(Instruction::MoveVal {
					offset: x.into(),
					factor: j as u8,
				}))
			}
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {value: -1, offset: None},
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(y))
			] | [
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal {value: -1, offset: None},
				Instruction::MovePtr(Offset::Relative(y))
			]
			if *x == -y
		)
	}
}
