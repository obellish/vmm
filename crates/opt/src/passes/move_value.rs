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
					value: j @ 0..=i8::MAX,
					offset: None,
				},
				Instruction::MovePtr(Offset::Relative(y)),
			]
			| [
				Instruction::IncVal {
					value: j @ 0..=i8::MAX,
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

				Some(Change::ReplaceOne(Instruction::MoveAndAddVal {
					offset: x.into(),
					factor: j as u8,
				}))
			}
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: Some(offset @ Offset::Relative(_)),
				},
			]
			| [
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: Some(offset @ Offset::Relative(_)),
				},
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::MoveAndAddVal {
				offset: *offset,
				factor: *value as u8,
			})),
			_ => None,
		}
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(y))
			] | [
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(Offset::Relative(x)),
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::MovePtr(Offset::Relative(y))
			]
			if *x == -y
		) || matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::IncVal {
					value: 0..=i8::MAX,
					offset: Some(Offset::Relative(_))
				}
			] | [
				Instruction::IncVal {
					value: 0..=i8::MAX,
					offset: Some(Offset::Relative(_))
				},
				Instruction::IncVal {
					value: -1,
					offset: None
				}
			]
		)
	}
}
