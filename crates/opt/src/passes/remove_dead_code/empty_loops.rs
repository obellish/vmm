use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Default)]
pub struct RemoveEmptyLoopsPass;

impl LoopPass for RemoveEmptyLoopsPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change> {
		self.should_run(loop_values).then_some(Change::Remove)
	}

	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		loop_values.is_empty()
	}
}
