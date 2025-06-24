use vmm_ir::{Instruction, ScaleAnd, SuperInstruction, Value};
use vmm_num::ops::WrappingAdd;
use vmm_utils::GetOrZero;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantChangeValOffsetPass;

impl PeepholePass for RemoveRedundantChangeValOffsetPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal { offset: x, .. },
				Instruction::SetVal { offset: y, .. },
			] if *x == *y => Some(Change::remove_offset(0)),
			[
				Instruction::SetVal {
					value: a,
					offset: x,
				},
				Instruction::IncVal {
					offset: y,
					value: Value::Constant(b),
				},
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(
				WrappingAdd::wrapping_add(a.get_or_zero(), *b),
				*x,
			))),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: x,
					..
				}),
				Instruction::SetVal { offset: y, value },
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(
				value.get_or_zero(),
				*x,
			))),
			[
				Instruction::FetchVal(x),
				Instruction::SetVal {
					value: None,
					offset: y,
				},
			] if *x == *y => Some(Change::remove_offset(1)),
			[
				Instruction::ReplaceVal(x),
				Instruction::SetVal {
					value: None,
					offset: y,
				},
			] if *x == *y => Some(Change::remove_offset(1)),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: x, .. },
				Instruction::IncVal { offset: y, .. }
			] | [
				Instruction::IncVal { offset: x, .. }
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Move,
						offset: x,
						..
					}),
				Instruction::SetVal { offset: y, .. }
			] | [
				Instruction::FetchVal(x) | Instruction::ReplaceVal(x),
				Instruction::SetVal {
					value: None,
					offset: y
				}
			]
			if *x == *y
		)
	}
}
