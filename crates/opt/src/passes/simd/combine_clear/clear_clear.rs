use vmm_ir::{Instruction, Offset, Simc};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CombineClearClearToSimdPass;

impl PeepholePass for CombineClearClearToSimdPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: None,
					offset: None,
				},
				Instruction::SetVal {
					value: None,
					offset: Some(Offset::Relative(1)),
				},
			] => {
				println!("made it (cc)");

				None
			}
			_ => None,
		}
	}
}
