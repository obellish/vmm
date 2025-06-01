use std::fmt::Debug;

use tracing::warn;
use vmm_ir::{Instruction, LoopInstruction};

use super::Change;

pub trait Pass: Debug {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool;

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}

impl<P> Pass for P
where
	P: Debug + PeepholePass,
{
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		let mut i = 0;
		let mut progress = false;

		while program.len() >= P::SIZE && i < program.len() - (P::SIZE - 1) {
			let window = &program[i..(P::SIZE + i)].to_vec();

			assert_eq!(window.len(), P::SIZE);

			if !P::should_run(self, window) {
				i += 1;
				continue;
			}

			let change = P::run_pass(self, window);

			let (changed, removed) = change
				.map(|c| c.apply(program, i, P::SIZE))
				.unwrap_or_default();

			i -= removed;

			if changed {
				progress = true;
			} else {
				i += 1;

				// If the pass changed state due to the current run, we don't want to warn
				if P::should_run(self, window) {
					warn!("pass {self:?}::should_ran was true but didn't make changes");
					tracing::trace!("{window:?}");
				}
			}
		}

		progress
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		<P as PeepholePass>::should_run_on_dyn_loop(self)
	}
}

pub trait PeepholePass {
	const SIZE: usize;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change>;

	#[allow(unused)]
	fn should_run(&self, window: &[Instruction]) -> bool {
		true
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}

impl<P> PeepholePass for P
where
	P: LoopPass,
{
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::Loop(LoopInstruction::Dynamic(instructions))] = window {
			<P as LoopPass>::run_pass(self, instructions)
		} else {
			None
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [Instruction::Loop(LoopInstruction::Dynamic(instrs) | LoopInstruction::IfNz(instrs))] = window else {
			return false;
		};

		<P as LoopPass>::should_run(self, instrs)
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}

pub trait LoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change>;

	#[allow(unused)]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		true
	}
}
