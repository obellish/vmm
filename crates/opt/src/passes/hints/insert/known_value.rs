use std::ops::RangeInclusive;

use vmm_ir::{CompilerHint, Instruction};

use crate::{Change, RangePeepholePass};

#[derive(Debug, Default)]
pub struct InsertKnownValueHintPass;

impl RangePeepholePass for InsertKnownValueHintPass {
	const RANGE: RangeInclusive<usize> = 2..=3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::Block(b), last] if !last.is_hint() => Some(Change::swap([
				Instruction::Block(b.clone()),
				CompilerHint::known_value(0).into(),
				last.clone(),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::Block(..), last]
			if !last.is_hint()
		)
	}
}
