use std::borrow::Cow;

use crate::{Change, Instruction, LoopPass, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct RemoveEmptyLoopsPass;

impl LoopPass for RemoveEmptyLoopsPass {
	fn run_pass(&self, loop_values: &[Instruction]) -> Option<Change> {
		loop_values.is_empty().then_some(Change::Remove)
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("remove empty loops")
	}
}
