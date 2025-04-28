use std::borrow::Cow;

use super::Change;
use crate::{ExecutionUnit, Instruction};

pub trait Pass {
	fn run_pass(&self, unit: &mut ExecutionUnit) -> bool;

	fn name(&self) -> Cow<'static, str>;
}

pub trait PeepholePass {
	const SIZE: usize;

	fn run_pass(&self, window: &[Instruction]) -> Change;

	fn name(&self) -> Cow<'static, str>;
}

impl<P: PeepholePass> Pass for P {
	fn run_pass(&self, unit: &mut ExecutionUnit) -> bool {
		let mut i = 0;
		let mut progress = false;

		while unit.program().len() >= P::SIZE && i < unit.program().len() - (P::SIZE - 1) {
			let window = &unit.program()[i..(P::SIZE + i)];

			let change = P::run_pass(self, window);

			let (changed, removed) = change.apply(unit.program_mut().as_raw(), i, P::SIZE);

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
		Cow::Owned(format!(
			"{} with window size of {}",
			<P as PeepholePass>::name(self),
			Self::SIZE
		))
	}
}
