use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};
use vmm_wrap::ops::WrappingMul;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeScaleValPass;

impl PeepholePass for OptimizeScaleValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(x),
					factor: a,
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Fetch,
					offset: Offset::Relative(y),
					factor: b,
				}),
			] if *x == *y => Some(Change::replace(Instruction::scale_val(
				WrappingMul::wrapping_mul(a, b),
			))),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: Offset::Relative(x),
					factor: a,
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(y),
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
					offset: Offset::Relative(x),
					factor,
				}),
				Instruction::TakeVal(Offset::Relative(y)),
			] if *x == -y => Some(Change::replace(Instruction::scale_val(*factor))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(x),
					..
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Fetch,
					offset: Offset::Relative(y),
					..
				})
			]
			if *x == *y
		) || matches!(
			window,
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset: Offset::Relative(x),
					..
				}),
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					offset: Offset::Relative(y),
					..
				}) | Instruction::TakeVal(Offset::Relative(y))
			]
			if *x == -y
		)
	}
}
