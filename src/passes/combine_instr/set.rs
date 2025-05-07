use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct CombineSetInstrPass;

impl PeepholePass for CombineSetInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Set(0), Instruction::Inc(y)] => {
				Some(Change::ReplaceOne(Instruction::Set(*y as u8)))
			}
			_ => None,
		}
	}
}
