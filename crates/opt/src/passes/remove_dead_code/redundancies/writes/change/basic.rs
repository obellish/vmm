use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};
use vmm_utils::GetOrZero as _;
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
				Instruction::SetVal {
					value,
					offset: None,
				},
			] => Some(Change::ReplaceOne(Instruction::set_val(
				value.get_or_zero(),
			))),
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
				dyn_loop @ Instruction::DynamicLoop(..),
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] => Some(Change::ReplaceOne(dyn_loop.clone())),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset,
					factor,
				}),
				Instruction::SetVal {
					offset: None,
					value: None,
				},
			] => Some(Change::ReplaceOne(Instruction::scale_and_move_val(
				*factor, *offset,
			))),
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
				Instruction::DynamicLoop(..)
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
			]
		)
	}
}
