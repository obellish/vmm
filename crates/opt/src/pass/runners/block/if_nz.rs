use std::fmt::{Debug, Formatter, Result as FmtResult};

use vmm_ir::{BlockInstruction, Instruction};

use super::BlockLength;
use crate::{Change, LoopPass, PeepholePass};

#[repr(transparent)]
pub struct IfNzRunner<P>(pub P);

impl<P: Debug> Debug for IfNzRunner<P> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<P: Default> Default for IfNzRunner<P> {
	fn default() -> Self {
		Self(P::default())
	}
}

impl<P: LoopPass> PeepholePass for IfNzRunner<P> {
	const SIZE: usize = 1;

	#[inline]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::Block(BlockInstruction::IfNz(instructions))] = window {
			self.0.run_pass(instructions)
		} else {
			None
		}
	}

	#[inline]
	fn should_run(&self, window: &[Instruction]) -> bool {
		let [Instruction::Block(BlockInstruction::IfNz(instrs))] = window else {
			return false;
		};

		let len = BlockLength::new(self.0.size_hint());

		match len {
			BlockLength::Unknown => {}
			BlockLength::Single(len) => {
				if instrs.len() != len {
					return false;
				}
			}
			BlockLength::LowerBound(len) => {
				if instrs.len() < len {
					return false;
				}
			}
			BlockLength::Range(range) => {
				if !range.contains(&instrs.len()) {
					return false;
				}
			}
		}

		self.0.should_run(instrs)
	}

	fn should_run_on_if(&self) -> bool {
		true
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}
}
