use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct InspectInstrPass {
	moves: usize,
	stays: usize,
}

impl PeepholePass for InspectInstrPass {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[lop @ Instruction::RawLoop(_)] => {
				// println!("{}", lop.might_move_ptr());

				// None
				if lop.might_move_ptr() {
					self.moves += 1;
				} else {
					self.stays += 1;
				}

				None
			}
			_ => None,
		}
	}

	fn should_run(&self, _window: &[Instruction]) -> bool {
		matches!(_window, [Instruction::RawLoop(_)])
	}
}

impl Drop for InspectInstrPass {
	fn drop(&mut self) {
		println!("loops that move ptr - {}", self.moves);
		println!("loops that don't move ptr - {}", self.stays);
	}
}
