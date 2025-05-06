use std::borrow::Cow;

use super::Change;
use crate::{ExecutionUnit, Instruction, Program};

pub trait Pass {
	fn run_pass(&self, unit: &mut Vec<Instruction>) -> bool;

	fn name(&self) -> Cow<'static, str>;

	fn should_run_on_loop(&self) -> bool {
		true
	}
}

pub trait PeepholePass {
	const SIZE: usize;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change>;

	fn name(&self) -> Cow<'static, str>;

	fn should_run_on_loop(&self) -> bool {
		true
	}
}

impl<P: PeepholePass> Pass for P {
	fn run_pass(&self, unit: &mut Vec<Instruction>) -> bool {
		let mut i = 0;
		let mut progress = false;

		while unit.len() >= P::SIZE && i < unit.len() - (P::SIZE - 1) {
			let window = &unit[i..(P::SIZE + i)];

			assert_eq!(window.len(), P::SIZE);

			let change = P::run_pass(self, window);

			let (changed, removed) = change
				.map(|c| c.apply(unit, i, P::SIZE))
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

	fn name(&self) -> Cow<'static, str> {
		<P as PeepholePass>::name(self)
	}

	fn should_run_on_loop(&self) -> bool {
		<P as PeepholePass>::should_run_on_loop(self)
	}
}
