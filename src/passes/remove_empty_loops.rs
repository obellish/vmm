use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct RemoveEmptyLoopsPass;

impl PeepholePass for RemoveEmptyLoopsPass {
	const SIZE: usize = 1;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		if window.first()?.is_empty() {
			Some(Change::Remove)
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("remove empty loops")
	}
}
