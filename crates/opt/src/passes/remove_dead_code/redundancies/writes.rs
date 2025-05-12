use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantWritesPass;

impl PeepholePass for RemoveRedundantWritesPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::IncVal(_), Instruction::SetVal(x)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(*x)))
			}
			[Instruction::SetVal(0), Instruction::IncVal(y)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(*y as u8)))
			}
			[Instruction::SetVal(x), Instruction::IncVal(y)] => Some(Change::ReplaceOne(
				Instruction::SetVal((*x as i8).wrapping_add(*y) as u8),
			)),
			_ => None,
		}
	}
}
