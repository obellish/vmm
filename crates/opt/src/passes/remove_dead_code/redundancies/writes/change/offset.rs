use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};
use vmm_utils::GetOrZero;
use vmm_wrap::ops::WrappingAdd;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantChangeValOffsetPass;

impl PeepholePass for RemoveRedundantChangeValOffsetPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					offset: Some(Offset::Relative(x)),
					..
				},
				Instruction::SetVal {
					offset: Some(Offset::Relative(y)),
					..
				},
			] if *x == *y => Some(Change::remove_offset(0)),
			[
				Instruction::SetVal {
					value: None,
					offset: Some(Offset::Relative(x)),
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					value,
				},
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(*value as u8, x))),
			[
				Instruction::SetVal {
					value: Some(a),
					offset: Some(Offset::Relative(x)),
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					value: b,
				},
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(
				WrappingAdd::wrapping_add(a.get(), *b),
				x,
			))),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(x),
					..
				}),
				Instruction::SetVal {
					offset: Some(Offset::Relative(y)),
					value,
				},
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(
				value.get_or_zero(),
				x,
			))),
			[
				Instruction::FetchVal(Offset::Relative(x)),
				Instruction::SetVal {
					value: None,
					offset: Some(Offset::Relative(y)),
				},
			] if *x == *y => Some(Change::remove_offset(1)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					offset: Some(Offset::Relative(x)),
					..
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					..
				}
			] | [
				Instruction::IncVal {
					offset: Some(Offset::Relative(x)),
					..
				} | Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(x),
					..
				}),
				Instruction::SetVal {
					offset: Some(Offset::Relative(y)),
					..
				}
			] | [
				Instruction::FetchVal(Offset::Relative(x)),
				Instruction::SetVal {
					value: None,
					offset: Some(Offset::Relative(y))
				}
			]
			if *x == *y
		)
	}
}
