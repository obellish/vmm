use vmm_ir::Offset;

use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeMoveValPass;

impl LoopPass for OptimizeMoveValPass {
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
				let x = *x;

				Some(Change::ReplaceOne(Instruction::scale_and_move_val_by(
					x, *j as u8,
				)))
			}
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: Some(Offset::Relative(x)),
				},
			]
			| [
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: Some(Offset::Relative(x)),
				},
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::scale_and_move_val_by(
				*x,
				*value as u8,
			))),
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
