use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct RemoveEmptyLoopsPass;

impl LoopPass for RemoveEmptyLoopsPass {
	fn run_pass(&mut self, _: &[Instruction]) -> Option<Change> {
		// Already verified by the should_run method, so just remove it
		Some(Change::Remove)
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		loop_values.is_empty()
	}
}
