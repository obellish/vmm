use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct RemoveEmptyLoopsPass;

impl LoopPass for RemoveEmptyLoopsPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		loop_values.is_empty().then_some(Change::Remove)
	}
}
