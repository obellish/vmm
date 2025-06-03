use std::fmt::{Debug, Formatter, Result as FmtResult};

use tracing::warn;
use vmm_ir::Instruction;

use crate::{Pass, PeepholePass};

#[repr(transparent)]
pub struct PeepholeRunner<P>(pub P);

impl<P: Debug> Debug for PeepholeRunner<P> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<P: Default> Default for PeepholeRunner<P> {
	fn default() -> Self {
		Self(P::default())
	}
}

impl<P> Pass for PeepholeRunner<P>
where
	P: Debug + PeepholePass,
{
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		let mut i = 0;
		let mut progress = false;

		while program.len() >= P::SIZE && i < program.len() - (P::SIZE - 1) {
			let window = program[i..(P::SIZE + i)].to_vec();

			assert_eq!(window.len(), P::SIZE);

			if !self.0.should_run(&window) {
				i += 1;
				continue;
			}

			let change = self.0.run_pass(&window);

			let (changed, removed) = change
				.map(|c| c.apply(program, i, P::SIZE))
				.unwrap_or_default();

			i -= removed;

			if changed {
				progress = true;
			} else {
				i += 1;

				if self.0.should_run(&window) {
					warn!(
						"pass {:?}::should_run was true but didn't make changes",
						self.0
					);
					tracing::debug!("{window:?}");
				}
			}
		}

		progress
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		self.0.should_run_on_dyn_loop()
	}

	fn should_run_on_if(&self) -> bool {
		self.0.should_run_on_if()
	}
}
