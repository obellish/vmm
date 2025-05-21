use vmm_ir::{Instruction, MoveBy};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseStackedInstrPass;

impl PeepholePass for CollapseStackedInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::IncVal(i1, None), Instruction::IncVal(i2, None)] if *i1 == -i2 => {
				Some(Change::Remove)
			}
			[Instruction::IncVal(i1, None), Instruction::IncVal(i2, None)] => Some(
				Change::ReplaceOne(Instruction::inc_val(i1.wrapping_add(*i2))),
			),
			[
				Instruction::MovePtr(MoveBy::Relative(i1)),
				Instruction::MovePtr(MoveBy::Relative(i2)),
			] if *i1 == -i2 => Some(Change::Remove),
			[
				Instruction::MovePtr(MoveBy::Relative(i1)),
				Instruction::MovePtr(MoveBy::Relative(i2)),
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
			[Instruction::IncVal(_, None), Instruction::IncVal(_, None)]
				| [
					Instruction::MovePtr(MoveBy::Relative(_)),
					Instruction::MovePtr(MoveBy::Relative(_))
				] | [Instruction::SetVal(_), Instruction::SetVal(_)]
		)
	}
}
