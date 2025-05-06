use std::borrow::Cow;

use crate::{Change, Instruction, LoopPass};

#[derive(Debug, Clone, Copy)]
pub struct SetZeroPass;

impl LoopPass for SetZeroPass {
	fn run_pass(&self, loop_values: &[Instruction]) -> Option<Change> {
		if let [Instruction::Add(-1)] = loop_values {
			Some(Change::ReplaceOne(Instruction::Set(0)))
		} else {
			None
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("set zero")
	}
}
