use std::ops::RangeInclusive;

use vmm_ir::{CompilerHint, Instruction};
use vmm_utils::GetOrZero as _;

use crate::{Change, RangePeepholePass};

#[derive(Debug, Default)]
pub struct InsertKnownValueHintPass;

impl RangePeepholePass for InsertKnownValueHintPass {
	const RANGE: RangeInclusive<usize> = 2..=3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal { value, offset: x },
				Instruction::Write { offset: y },
				last,
			] if !last.is_hint() => Some(Change::swap([
				Instruction::set_val_at(value.get_or_zero(), x),
				Instruction::write_once_at(y),
				CompilerHint::known_value_at(value.get_or_zero(), x).into(),
				last.clone(),
			])),
			[Instruction::Block(b), last] if !last.is_hint() => Some(Change::swap([
				Instruction::Block(b.clone()),
				CompilerHint::known_value(0).into(),
				last.clone(),
			])),
			[
				Instruction::Hint(hint @ CompilerHint::KnownValue { .. }),
				Instruction::Write { offset },
				last,
			] if !last.is_hint() => Some(Change::swap([
				Instruction::Hint(*hint),
				Instruction::write_once_at(offset),
				Instruction::Hint(*hint),
				last.clone(),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { .. } | Instruction::Hint(CompilerHint::KnownValue { .. }),
				Instruction::Write { .. },
				last
			] | [Instruction::Block(..), last]
			if !last.is_hint()
		)
	}
}
