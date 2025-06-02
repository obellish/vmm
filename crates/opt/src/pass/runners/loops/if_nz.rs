use std::fmt::{Debug, Formatter, Result as FmtResult};

use vmm_ir::{BlockInstruction, Instruction};

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

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::Block(BlockInstruction::IfNz(instructions))] = window {
			self.0.run_pass(instructions)
		} else {
			None
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [Instruction::Block(BlockInstruction::IfNz(instrs))] = window else {
			return false;
		};

		self.0.should_run(instrs)
	}

	fn should_run_on_if(&self) -> bool {
		true
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}
}
