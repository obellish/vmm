use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct RemoveRedundantWritesPass;

impl PeepholePass for RemoveRedundantWritesPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Inc(_), Instruction::Set(0)] => {
				Some(Change::ReplaceOne(Instruction::Set(0)))
			}
			_ => None,
		}
	}
}
