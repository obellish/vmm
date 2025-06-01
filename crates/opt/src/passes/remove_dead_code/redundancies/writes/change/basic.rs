use vmm_ir::{Instruction, LoopInstruction, ScaleAnd, SuperInstruction};
use vmm_wrap::Wrapping;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantChangeValBasicPass;

impl PeepholePass for RemoveRedundantChangeValBasicPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::SetVal { offset: None, .. },
			] => Some(Change::RemoveOffset(0)),
			[
				Instruction::SetVal {
					value: None,
					offset: None,
				},
				Instruction::IncVal {
					value: y,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::set_val(*y as u8))),
			[
				Instruction::SetVal {
					offset: None,
					value: Some(x),
				},
				Instruction::IncVal {
					value: y,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::set_val(Wrapping::add(
				x.get(),
				*y,
			)))),
			[
				Instruction::IncVal { offset: None, .. } | Instruction::SetVal { offset: None, .. },
				Instruction::Read,
			] => Some(Change::ReplaceOne(Instruction::read())),
			[
				Instruction::Loop(LoopInstruction::Dynamic(..) | LoopInstruction::IfNz(..))
				| Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					..
				}),
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] => Some(Change::RemoveOffset(1)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: None, .. },
				Instruction::IncVal { offset: None, .. } | Instruction::Read
			] | [
				Instruction::Loop(LoopInstruction::Dynamic(..) | LoopInstruction::IfNz(..))
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Move,
						..
					}),
				Instruction::SetVal {
					offset: None,
					value: None
				}
			] | [
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Fetch,
					..
				}) | Instruction::IncVal { offset: None, .. },
				Instruction::Read
			] | [
				Instruction::IncVal { offset: None, .. },
				Instruction::SetVal { offset: None, .. }
			]
		)
	}
}
