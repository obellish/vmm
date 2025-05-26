use std::{
	fmt::Debug,
	ops::{Bound, RangeBounds},
};

use tracing::{trace, warn};
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

		let range = self.window_sizes();

		let start = match range.start_bound() {
			Bound::Unbounded => unreachable!(),
			Bound::Excluded(e) | Bound::Included(e) => *e,
		};

		let end = match range.end_bound() {
			Bound::Unbounded => unreachable!(),
			Bound::Excluded(e) => *e,
			Bound::Included(e) => *e - 1,
		};

		drop(range);

		for size in start..end {
			trace!("running pass {self:?} with size {size}");
			while program.len() >= size && i < program.len() - (size - 1) {
				let window = &program[i..(size + i)].to_vec();

				assert_eq!(window.len(), size);

				if !P::should_run(self, window) {
					i += 1;
					continue;
				}

				let change = P::run_pass(self, window);

				let (changed, removed) = change
					.map(|c| c.apply(program, i, size))
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

	#[allow(unused)]
	fn should_run(&self, window: &[Instruction]) -> bool {
		true
	}

	fn window_sizes(&self) -> impl RangeBounds<usize> {
		Self::SIZE..(Self::SIZE + 1)
	}

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
		if let [Instruction::DynamicLoop(instructions)] = window {
			<P as LoopPass>::run_pass(self, instructions)
		} else {
			None
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		let [Instruction::DynamicLoop(instrs)] = window else {
			return false;
		};

		<P as LoopPass>::should_run(self, instrs)
	}

	fn should_run_on_loop(&self) -> bool {
		true
	}
}

pub trait LoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change>;

	fn should_run(&self, _loop_values: &[Instruction]) -> bool {
		true
	}
}
