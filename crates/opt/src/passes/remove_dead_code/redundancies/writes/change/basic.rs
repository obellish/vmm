use vmm_ir::{BlockInstruction, Instruction, Offset, ScaleAnd, SuperInstruction, Value};
use vmm_num::ops::WrappingAdd;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantChangeValBasicPass;

impl PeepholePass for RemoveRedundantChangeValBasicPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					offset: Offset(0),
					value: x,
				},
				Instruction::IncVal {
					value: Value::Constant(y),
					offset: Offset(0),
				},
			] => Some(Change::replace(Instruction::set_val(
				WrappingAdd::wrapping_add(x.get_or_zero(), *y),
			))),
			[
				Instruction::IncVal {
					offset: Offset(0), ..
				}
				| Instruction::SetVal {
					offset: Offset(0), ..
				},
				Instruction::Read,
			]
			| [
				Instruction::IncVal {
					offset: Offset(0), ..
				}
				| Instruction::ScaleVal { .. },
				Instruction::SetVal {
					offset: Offset(0), ..
				},
			] => Some(Change::remove_offset(0)),
			[
				Instruction::Block(BlockInstruction::DynamicLoop(..) | BlockInstruction::IfNz(..))
				| Instruction::Super(
					SuperInstruction::ScaleAnd {
						action: ScaleAnd::Move,
						..
					}
					| SuperInstruction::SetUntilZero { .. },
				)
				| Instruction::SubCell { .. }
				| Instruction::MoveVal(..),
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
			]
			| [
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
				Instruction::SubCell { .. } | Instruction::ScaleVal { .. },
			] => Some(Change::remove_offset(1)),
			[
				Instruction::TakeVal(offset)
				| Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset,
					..
				}),
				Instruction::SetVal {
					value,
					offset: Offset(0),
				},
			] => Some(Change::swap([
				Instruction::clear_val(),
				Instruction::move_ptr(*offset),
				Instruction::set_val(value.get_or_zero()),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					offset: Offset(0),
					..
				},
				Instruction::IncVal {
					offset: Offset(0),
					value: Value::Constant(..)
				} | Instruction::Read
			] | [
				Instruction::Block(BlockInstruction::DynamicLoop(..) | BlockInstruction::IfNz(..))
					| Instruction::Super(
						SuperInstruction::ScaleAnd {
							action: ScaleAnd::Move,
							..
						} | SuperInstruction::SetUntilZero { .. }
					) | Instruction::SubCell { .. }
					| Instruction::MoveVal(..),
				Instruction::SetVal {
					offset: Offset(0),
					value: None
				}
			] | [
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Fetch,
					..
				}) | Instruction::IncVal {
					offset: Offset(0),
					..
				},
				Instruction::Read
			] | [
				Instruction::IncVal {
					offset: Offset(0),
					..
				} | Instruction::TakeVal(..)
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Take,
						..
					}) | Instruction::ScaleVal { .. },
				Instruction::SetVal {
					offset: Offset(0),
					..
				}
			] | [
				Instruction::SetVal {
					value: None,
					offset: Offset(0)
				},
				Instruction::SubCell { .. } | Instruction::ScaleVal { .. }
			]
		)
	}
}
