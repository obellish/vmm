use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct CombineSetInstrPass;

impl PeepholePass for CombineSetInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::SetVal(0), Instruction::IncVal(y)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(*y as u8)))
			}
			[Instruction::SetVal(x), Instruction::SetVal(y)] if (*x as i8) == -(*y as i8) => {
				Some(Change::Remove)
			}
			[Instruction::SetVal(x), Instruction::SetVal(y)] => Some(Change::ReplaceOne(
				Instruction::SetVal(((*x as i8).wrapping_add(*y as i8)) as u8),
			)),
			_ => None,
		}
	}
}
