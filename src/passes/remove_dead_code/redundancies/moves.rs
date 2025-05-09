use crate::{Change, Instruction, PeepholePass, StackedInstruction};

#[derive(Debug, Default)]
pub struct RemoveRedundantMovesPass;

impl PeepholePass for RemoveRedundantMovesPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal(x),
				Instruction::MoveVal { offset, multiplier },
			] => Some(Change::Replace(vec![
				Instruction::SetVal(0),
				StackedInstruction::MovePtr(*offset).into(),
				Instruction::SetVal(x.wrapping_mul(*multiplier)),
				StackedInstruction::MovePtr(-*offset).into(),
			])),
			_ => None,
		}
	}
}
