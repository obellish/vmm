use vmm_ir::{Instruction, ScaleAnd, SuperInstruction};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct RemoveRedundantScaleValInstrBasicPass;

impl PeepholePass for RemoveRedundantScaleValInstrBasicPass {
	const SIZE: usize = 1;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[Instruction::ScaleVal { factor: 0 }] => {
				Some(Change::replace(Instruction::clear_val()))
			}
			[Instruction::ScaleVal { factor: 1 }] => Some(Change::remove()),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Take,
					offset,
					factor: 0,
				}),
			] => Some(Change::swap([
				Instruction::clear_val(),
				Instruction::move_ptr(offset),
			])),
			[
				Instruction::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					factor: 0,
					..
				}),
			] => Some(Change::replace(Instruction::clear_val())),
			_ => None,
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[Instruction::ScaleVal { factor: 0 | 1 }
				| Instruction::Super(SuperInstruction::ScaleAnd { factor: 0, .. })]
		)
	}
}
