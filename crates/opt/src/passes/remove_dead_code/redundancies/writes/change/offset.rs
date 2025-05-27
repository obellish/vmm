use vmm_ir::{Instruction, Offset};
use vmm_utils::GetOrZero;
use vmm_wrap::Wrapping;

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
					value,
				},
			] if *x == *y => Some(Change::ReplaceOne(Instruction::set_val_at(
				value.get_or_zero(),
				x,
			))),
			[
				Instruction::SetVal {
					value: None,
					offset: Some(Offset::Relative(x)),
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					value,
				},
			] if *x == *y => Some(Change::ReplaceOne(Instruction::set_val_at(*value as u8, x))),
			[
				Instruction::SetVal {
					value: Some(a),
					offset: Some(Offset::Relative(x)),
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					value: b,
				},
			] if *x == *y => Some(Change::ReplaceOne(Instruction::set_val_at(
				(Wrapping(a.get()) + *b).0,
				x,
			))),
			[
				Instruction::ScaleAndMoveVal {
					offset: Offset::Relative(x),
					..
				},
				Instruction::SetVal {
					offset: Some(Offset::Relative(y)),
					value,
				},
			] if *x == *y => Some(Change::ReplaceOne(Instruction::set_val_at(
				value.get_or_zero(),
				x,
			))),
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
				} | Instruction::ScaleAndMoveVal {
					offset: Offset::Relative(x),
					..
				},
				Instruction::SetVal {
					offset: Some(Offset::Relative(y)),
					..
				}
			]
			if *x == *y
		)
	}
}
