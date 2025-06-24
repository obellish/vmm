use vmm_ir::{Instruction, Offset, WriteInstruction};
use vmm_num::ops::WrappingAdd;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeWriteBasicPass;

impl PeepholePass for OptimizeWriteBasicPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Write(WriteInstruction::CellAndSet {
					count, offset: x, ..
				}),
				Instruction::SetVal { value, offset: y },
			] if *x == *y => Some(Change::replace(Instruction::write_many_and_set_at(
				*count,
				x,
				value.get_or_zero(),
			))),
			[
				Instruction::Write(WriteInstruction::Cell { count, offset: x }),
				Instruction::SetVal { value, offset: y },
			] if *x == *y => Some(Change::replace(Instruction::write_many_and_set_at(
				*count,
				x,
				value.get_or_zero(),
			))),
			[
				Instruction::Write(WriteInstruction::Byte(x)),
				Instruction::IncVal {
					offset: Offset(0),
					value: y,
				},
			] => Some(Change::swap([
				Instruction::write_byte(*x),
				Instruction::set_val(WrappingAdd::wrapping_add(x, y)),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Write(
					WriteInstruction::CellAndSet { offset: x, .. }
						| WriteInstruction::Cell { offset: x, .. }
				),
				Instruction::SetVal { offset: y, .. }
			] | [
				Instruction::Write(WriteInstruction::CellAndSet { offset: x, .. }),
				Instruction::IncVal { offset: y, .. }
			]
			if *x == *y
		) || matches!(
			window,
			[
				Instruction::Write(WriteInstruction::Byte(..)),
				Instruction::IncVal {
					offset: Offset(0),
					..
				}
			]
		)
	}
}
