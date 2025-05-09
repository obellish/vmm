use crate::{Change, Instruction::Stacked, PeepholePass, StackedInstruction};

#[derive(Debug, Default, Clone, Copy)]
pub struct CombineStackedInstrPass;

impl PeepholePass for CombineStackedInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[crate::Instruction]) -> Option<Change> {
		match window {
			[
				Stacked(StackedInstruction::IncVal(i1)),
				Stacked(StackedInstruction::IncVal(i2)),
			] if *i1 == -*i2 => Some(Change::Remove),
			[
				Stacked(StackedInstruction::IncVal(i1)),
				Stacked(StackedInstruction::IncVal(i2)),
			] => Some(Change::ReplaceOne(Stacked(StackedInstruction::IncVal(
				i1.wrapping_add(*i2),
			)))),
			[
				Stacked(StackedInstruction::MovePtr(i1)),
				Stacked(StackedInstruction::MovePtr(i2)),
			] if *i1 == -*i2 => Some(Change::Remove),
			[
				Stacked(StackedInstruction::MovePtr(i1)),
				Stacked(StackedInstruction::MovePtr(i2)),
			] => Some(Change::ReplaceOne(Stacked(StackedInstruction::MovePtr(
				i1.wrapping_add(*i2),
			)))),
			_ => None,
		}
	}
}
