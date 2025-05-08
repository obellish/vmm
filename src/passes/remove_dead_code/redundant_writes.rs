use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default, Clone, Copy)]
pub struct RemoveRedundantWritesPass;

impl PeepholePass for RemoveRedundantWritesPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::IncVal(_), Instruction::SetVal(0)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(0)))
			}
			_ => None,
		}
	}
}
