use vmm_ir::{Instruction, Offset, ScaleAnd, SuperInstruction};
use vmm_num::ops::WrappingMul;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSuperInstrPass;

impl PeepholePass for OptimizeSuperInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: None,
					offset: Offset(0),
				},
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset,
					..
				}),
			] => Some(Change::swap([
				Instruction::clear_val(),
				Instruction::move_ptr(offset),
				Instruction::clear_val(),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
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
				Instruction::move_ptr(offset),
				Instruction::set_val(value.get_or_zero()),
			])),
			[
				Instruction::ScaleVal { factor: x },
				Instruction::Super(SuperInstruction::ScaleAnd {
					action,
					offset,
					factor: y,
				}),
			] => Some(Change::replace(
				SuperInstruction::ScaleAnd {
					action: *action,
					offset: *offset,
					factor: WrappingMul::wrapping_mul(x, y),
				}
				.into(),
			)),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal {
					offset: Offset(0),
					value: None,
				},
				Instruction::TakeVal(..)
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Take | ScaleAnd::Move,
						..
					})
			] | [
				Instruction::TakeVal(..)
					| Instruction::Super(SuperInstruction::ScaleAnd {
						action: ScaleAnd::Take,
						..
					}),
				Instruction::SetVal {
					offset: Offset(0),
					..
				}
			] | [
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					..
				}),
				Instruction::ScaleVal { .. }
			] | [
				Instruction::ScaleVal { .. },
				Instruction::Super(SuperInstruction::ScaleAnd { .. })
			]
		)
	}
}
