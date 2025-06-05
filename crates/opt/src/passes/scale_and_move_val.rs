use vmm_ir::Instruction;

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeScaleAndMoveValPass;

impl LoopPass for OptimizeScaleAndMoveValPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value: j @ 0..=i8::MAX,
					offset: None,
				},
				Instruction::MovePtr(y),
			]
			| [
				Instruction::IncVal {
					value: j @ 0..=i8::MAX,
					offset: None,
				},
				Instruction::MovePtr(y),
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::MovePtr(x),
			] if *x == -y => {
				let x = *x;

				Some(Change::replace(Instruction::scale_and_move_val(
					*j as u8, x,
				)))
			}
			[
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: Some(x),
				},
			]
			| [
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: Some(x),
				},
				Instruction::IncVal {
					value: -1,
					offset: None,
				},
			] => Some(Change::replace(Instruction::scale_and_move_val(
				*value as u8,
				*x,
			))),
			_ => None,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(2, Some(4))
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::MovePtr(x),
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(y)
			] | [
				Instruction::IncVal { offset: None, .. },
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value: -1,
					offset: None
				},
				Instruction::MovePtr(y)
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
					offset: Some(_)
				}
			] | [
				Instruction::IncVal {
					value: 0..=i8::MAX,
					offset: Some(_)
				},
				Instruction::IncVal {
					value: -1,
					offset: None
				}
			]
		)
	}
}
