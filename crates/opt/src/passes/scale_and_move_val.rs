use vmm_ir::{Instruction, Offset};

use crate::{Change, LoopPass};

#[derive(Debug, Default)]
pub struct OptimizeScaleAndMoveValPass;

impl LoopPass for OptimizeScaleAndMoveValPass {
	#[inline]
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		match loop_values {
			[
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
				},
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value: j @ 0..=i8::MAX,
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
			]
			| [
				Instruction::IncVal {
					value: j @ 0..=i8::MAX,
					offset: Offset(0),
				},
				Instruction::MovePtr(y),
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
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
					offset: Offset(0),
				},
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: x,
				},
			]
			| [
				Instruction::IncVal {
					value: value @ 0..=i8::MAX,
					offset: x,
				},
				Instruction::IncVal {
					value: -1,
					offset: Offset(0),
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

	#[inline]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: Offset(0)
				},
				Instruction::MovePtr(x),
				Instruction::IncVal { offset: Offset(0), .. },
				Instruction::MovePtr(y)
			] | [
				Instruction::IncVal { offset: Offset(0), .. },
				Instruction::MovePtr(x),
				Instruction::IncVal {
					value: -1,
					offset: Offset(0)
				},
				Instruction::MovePtr(y)
			]
			if *x == -y
		) || matches!(
			loop_values,
			[
				Instruction::IncVal {
					value: -1,
					offset: Offset(0)
				},
				Instruction::IncVal {
					value: 0..=i8::MAX,
					..
				}
			] | [
				Instruction::IncVal {
					value: 0..=i8::MAX,
					..
				},
				Instruction::IncVal {
					value: -1,
					offset: Offset(0)
				}
			]
		)
	}
}
