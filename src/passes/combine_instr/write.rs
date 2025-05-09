use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default, Clone, Copy)]
pub struct CombineWriteInstrPass;

impl PeepholePass for CombineWriteInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Write(x), Instruction::Write(y)] => {
				Some(Change::ReplaceOne(Instruction::Write(*x + *y)))
			}
			_ => None,
		}
	}
}
