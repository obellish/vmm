use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnreachableLoopsPass;

impl PeepholePass for RemoveUnreachableLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::SetVal(0), Instruction::RawLoop(..)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(0)))
			}
			_ => None,
		}
	}
}
