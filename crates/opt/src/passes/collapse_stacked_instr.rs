use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseStackedInstrPass;

impl PeepholePass for CollapseStackedInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::IncVal(i1), Instruction::IncVal(i2)] if *i1 == -i2 => {
				Some(Change::Remove)
			}
			[Instruction::IncVal(i1), Instruction::IncVal(i2)] => Some(Change::ReplaceOne(
				Instruction::IncVal(i1.wrapping_add(*i2)),
			)),
			[Instruction::MovePtr(i1), Instruction::MovePtr(i2)] if *i1 == -i2 => {
				Some(Change::Remove)
			}
			[Instruction::MovePtr(i1), Instruction::MovePtr(i2)] => Some(Change::ReplaceOne(
				Instruction::MovePtr(i1.wrapping_add(*i2)),
			)),
			[Instruction::SetVal(_), Instruction::SetVal(x)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(*x)))
			}
			_ => None,
		}
	}
}
