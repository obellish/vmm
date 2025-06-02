use vmm_ir::{BlockInstruction, Instruction, ScaleAnd, SuperInstruction};
use vmm_utils::GetOrZero as _;
use vmm_wrap::ops::WrappingAdd;

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
			] => Some(Change::ReplaceOne(Instruction::set_val(
				WrappingAdd::wrapping_add(x.get(), *y),
			))),
			[
				Instruction::IncVal { offset: None, .. } | Instruction::SetVal { offset: None, .. },
				Instruction::Read,
			] => Some(Change::ReplaceOne(Instruction::read())),
			[
				Instruction::Block(BlockInstruction::DynamicLoop(..) | BlockInstruction::IfNz(..))
				| Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					..
				})
				| Instruction::SubCell { .. }
				| Instruction::MoveVal(..),
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] => Some(Change::RemoveOffset(1)),
			[
				Instruction::TakeVal(offset)
				| Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset,
					..
				}),
				Instruction::SetVal {
					value,
					offset: None,
				},
			] => Some(Change::Replace(vec![
				Instruction::clear_val(),
				Instruction::move_ptr(*offset),
				Instruction::set_val(value.get_or_zero()),
			])),
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
				Instruction::Block(BlockInstruction::DynamicLoop(..) | BlockInstruction::IfNz(..))
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Move,
						..
					}) | Instruction::SubCell { .. }
					| Instruction::MoveVal(..),
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
				Instruction::IncVal { offset: None, .. }
					| Instruction::TakeVal(..)
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Take,
						..
					}),
				Instruction::SetVal { offset: None, .. }
			] | [
				Instruction::SetVal {
					value: None,
					offset: None
				},
				Instruction::SubCell { .. }
			]
		)
	}
}
