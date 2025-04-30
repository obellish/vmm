use std::{borrow::Cow, fmt::Debug};

use super::Change;
use crate::{ExecutionUnit, Instruction};

pub trait Pass {
	type State;

	fn run_pass(&mut self, unit: &mut ExecutionUnit, state: Self::State) -> bool;

	fn name(&self) -> Cow<'static, str>;
}

pub trait PeepholePass {
	type State: Clone;

	const SIZE: usize;

	fn run_pass(&mut self, window: &[Instruction], state: Self::State) -> Option<Change>;

	fn name(&self) -> Cow<'static, str>;
}

impl<P> Pass for P
where
	P: Debug + PeepholePass,
{
	type State = P::State;

	fn run_pass(&mut self, unit: &mut ExecutionUnit, state: Self::State) -> bool {
		let mut i = 0;
		let mut progress = false;

		while unit.program().len() >= P::SIZE && i < unit.program().len() - (P::SIZE - 1) {
			let window = &unit.program()[i..(P::SIZE + i)];

			assert_eq!(window.len(), P::SIZE);

			let change = P::run_pass(self, window, state.clone());

			let (changed, removed) = change
				.map(|c| c.apply(unit.program_mut().as_raw(), i, P::SIZE))
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
		Cow::Owned(format!(
			"{} with window size of {}",
			<P as PeepholePass>::name(self),
			Self::SIZE
		))
	}
}
