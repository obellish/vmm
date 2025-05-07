use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct CombineSetInstrPass;

impl PeepholePass for CombineSetInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Set(0), Instruction::Inc(y)] => {
				Some(Change::ReplaceOne(Instruction::Set(*y as u8)))
			}
			[Instruction::Set(x), Instruction::Set(y)] if (*x as i8) == -(*y as i8) => {
				Some(Change::Remove)
			}
			[Instruction::Set(x), Instruction::Set(y)] => Some(Change::ReplaceOne(
				Instruction::Set(((*x as i8).wrapping_add(*y as i8)) as u8),
			)),
			_ => None,
		}
	}
}
