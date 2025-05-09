use std::fmt::Debug;

use vmm_ir::Instruction;

use super::Change;

pub trait Pass: Debug {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool;

	fn should_run_on_loop(&self) -> bool {
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
			let window = &program[i..(P::SIZE + i)];

			assert_eq!(window.len(), P::SIZE);

			let change = P::run_pass(self, window);

			let (changed, removed) = change
				.map(|c| c.apply(program, i, P::SIZE))
				.unwrap_or_default();

			i -= removed;

			if changed {
				progress = true;
			} else {
				i += 1;
			}
		}

		progress
	}

	fn should_run_on_loop(&self) -> bool {
		<P as PeepholePass>::should_run_on_loop(self)
	}
}

pub trait PeepholePass {
	const SIZE: usize;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change>;

	fn should_run_on_loop(&self) -> bool {
		true
	}
}

impl<P> PeepholePass for P
where
	P: LoopPass,
{
	const SIZE: usize = 1;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::RawLoop(instructions)] = window {
			<P as LoopPass>::run_pass(self, instructions)
		} else {
			None
		}
	}

	fn should_run_on_loop(&self) -> bool {
		true
	}
}

pub trait LoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change>;
}
