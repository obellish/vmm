use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct UnrollScaleAndPass;

impl PeepholePass for UnrollScaleAndPass {
	const SIZE: usize = 1;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action,
					factor: 1,
					offset,
				}),
			] if !matches!(action, ScaleAnd::Set(..)) => Some(Change::replace(match action {
				ScaleAnd::Fetch => Instruction::fetch_val(*offset),
				ScaleAnd::Move => Instruction::move_val(*offset),
				ScaleAnd::Take => Instruction::take_val(*offset),
				_ => return None,
			})),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Set(value),
					factor: 1,
					offset,
				}),
			] => Some(Change::swap([
				Instruction::move_val(*offset),
				Instruction::set_val(value.get()),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset,
					factor,
				}),
			] => Some(Change::swap([
				Instruction::scale_val(*factor),
				Instruction::move_val(offset),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset,
					factor,
				}),
			] => Some(Change::swap([
				Instruction::scale_val(*factor),
				Instruction::take_val(offset),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Set(value),
					offset,
					factor,
				}),
			] => Some(Change::swap([
				Instruction::scale_val(*factor),
				Instruction::move_val(offset),
				Instruction::set_val(value.get_or_zero()),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::Super(SuperInstruction::ScaleAnd { .. })]
		)
	}
}
