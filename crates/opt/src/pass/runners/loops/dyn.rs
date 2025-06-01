use std::fmt::{Debug, Formatter, Result as FmtResult};

use vmm_ir::{Instruction, LoopInstruction};

use crate::{Change, LoopPass, PeepholePass};

#[repr(transparent)]
pub struct DynamicLoopRunner<P>(pub P);

impl<P: Debug> Debug for DynamicLoopRunner<P> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<P: Default> Default for DynamicLoopRunner<P> {
	fn default() -> Self {
		Self(P::default())
	}
}

impl<P: LoopPass> PeepholePass for DynamicLoopRunner<P> {
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::Loop(LoopInstruction::Dynamic(instructions))] = window {
			self.0.run_pass(instructions)
		} else {
			None
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [Instruction::Loop(LoopInstruction::Dynamic(instrs))] = window else {
			return false;
		};

		self.0.should_run(instrs)
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}
