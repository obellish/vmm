use vmm_ir::{Instruction, WriteInstruction};
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeWriteConstPass;

impl PeepholePass for OptimizeWriteConstPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal { value, offset: x },
				Instruction::Write(WriteInstruction::CellAndSet {
					count: 1,
					value: None,
					offset: y,
				}),
			] if *x == *y => Some(Change::swap([
				Instruction::clear_val_at(x),
				Instruction::write_byte(value.get_or_zero()),
			])),
			[
				Instruction::SetVal { value, offset: x },
				Instruction::Write(WriteInstruction::Cell {
					offset: y,
					count: 1,
				}),
			] if *x == *y => Some(Change::swap([
				Instruction::write_byte(value.get_or_zero()),
				Instruction::set_val_at(value.get_or_zero(), x),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: x, .. },
				Instruction::Write(
					WriteInstruction::CellAndSet {
						offset: y,
						value: None,
						..
					} | WriteInstruction::Cell { offset: y, .. }
				)
			]
			if *x == *y
		) || matches!(
			window,
			[
				Instruction::Write(WriteInstruction::Byte(..) | WriteInstruction::Bytes(..)),
				Instruction::Write(WriteInstruction::Byte(..) | WriteInstruction::Bytes(..))
			]
		)
	}
}
