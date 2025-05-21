use vmm_ir::{Instruction, Offset};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseStackedInstrPass;

impl PeepholePass for CollapseStackedInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: i1,
					offset: None,
				},
				Instruction::IncVal {
					value: i2,
					offset: None,
				},
			] if *i1 == -i2 => Some(Change::Remove),
			[
				Instruction::IncVal {
					value: i1,
					offset: None,
				},
				Instruction::IncVal {
					value: i2,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::inc_val(
				i1.wrapping_add(*i2),
			))),
			[
				Instruction::MovePtr(Offset::Relative(i1)),
				Instruction::MovePtr(Offset::Relative(i2)),
			] if *i1 == -i2 => Some(Change::Remove),
			[
				Instruction::MovePtr(Offset::Relative(i1)),
				Instruction::MovePtr(Offset::Relative(i2)),
			] => Some(Change::ReplaceOne(Instruction::MovePtr(
				i1.wrapping_add(*i2).into(),
			))),
			[Instruction::SetVal(_), Instruction::SetVal(x)] => {
				Some(Change::ReplaceOne(Instruction::SetVal(*x)))
			}
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::IncVal { offset: None, .. }
			] | [
				Instruction::MovePtr(Offset::Relative(_)),
				Instruction::MovePtr(Offset::Relative(_))
			] | [Instruction::SetVal(_), Instruction::SetVal(_)]
		)
	}
}
