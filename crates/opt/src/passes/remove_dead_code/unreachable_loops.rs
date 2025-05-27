use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveUnreachableLoopsPass;

impl PeepholePass for RemoveUnreachableLoopsPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[i, Instruction::DynamicLoop(..)] if i.is_zeroing_cell() => {
				Some(Change::ReplaceOne(i.clone()))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(window, [i, Instruction::DynamicLoop(..)] if i.is_zeroing_cell())
	}
}
