use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};
use vmm_num::ops::WrappingMul;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeScaleValPass;

impl PeepholePass for OptimizeScaleValPass {
	const SIZE: usize = 2;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: x,
					factor: a,
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Fetch,
					offset: y,
					factor: b,
				}),
			] if *x == *y => Some(Change::replace(Instruction::scale_val(
				WrappingMul::wrapping_mul(a, b),
			))),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: x,
					factor: a,
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: y,
					factor: b,
				}),
			] if *x == -y => Some(Change::swap([
				Instruction::scale_val(WrappingMul::wrapping_mul(a, b)),
				Instruction::move_ptr(*x),
				Instruction::clear_val(),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: x,
					factor,
				}),
				Instruction::TakeVal(y),
			] if *x == -y => Some(Change::replace(Instruction::scale_val(*factor))),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: x,
					factor,
				}),
				Instruction::MoveVal(y),
			] if *x == -y => Some(Change::swap([
				Instruction::scale_val(*factor),
				Instruction::move_ptr(x),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: x,
					factor: a,
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Set(value),
					offset: y,
					factor: b,
				}),
			] if *x == -y => Some(Change::swap([
				Instruction::scale_val(WrappingMul::wrapping_mul(a, b)),
				Instruction::move_ptr(x),
				Instruction::set_val(value.get()),
			])),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: x,
					..
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Fetch,
					offset: y,
					..
				})
			]
			if *x == *y
		) || matches!(
			window,
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: x,
					..
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move | ScaleAnd::Set(..),
					offset: y,
					..
				}) | Instruction::TakeVal(y)
					| Instruction::MoveVal(y)
			]
			if *x == -y
		)
	}
}
