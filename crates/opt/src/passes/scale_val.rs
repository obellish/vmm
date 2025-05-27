use vmm_ir::{Instruction, Offset};
use vmm_wrap::Wrapping;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ScaleValPass;

impl PeepholePass for ScaleValPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::ScaleAndMoveVal {
					offset: Offset::Relative(x),
					factor: a,
				},
				Instruction::FetchAndScaleVal {
					offset: Offset::Relative(y),
					factor: b,
				},
			] if *x == *y => Some(Change::ReplaceOne(Instruction::scale_val(
				(Wrapping(*a) * *b).0,
			))),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::ScaleAndMoveVal {
					offset: Offset::Relative(x),
					..
				},
				Instruction::FetchAndScaleVal {
					offset: Offset::Relative(y),
					..
				},
			]
			if *x == *y
		)
	}
}
