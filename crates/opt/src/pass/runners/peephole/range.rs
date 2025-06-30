use std::fmt::{Debug, Formatter, Result as FmtResult};

use tracing::warn;
use vmm_ir::Instruction;

use crate::{Pass, RangePeepholePass};

#[repr(transparent)]
pub struct RangePeepholeRunner<P>(pub P);

impl<P: Debug> Debug for RangePeepholeRunner<P> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<P: Default> Default for RangePeepholeRunner<P> {
	fn default() -> Self {
		Self(P::default())
	}
}

impl<P> Pass for RangePeepholeRunner<P>
where
	P: Debug + RangePeepholePass,
{
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool {
		let mut i = 0;
		let mut progress = false;

		while program.len() >= *P::RANGE.end() && i < program.len() - (P::RANGE.end() - 1) {
			for limit in P::RANGE {
				let window = program[i..(limit + i)].to_vec();

				assert!(P::RANGE.contains(&window.len()));

				if !self.0.should_run(&window) {
					continue;
				}

				let change = self.0.run_pass(&window);

				let (changed, removed) = change
					.map(|c| c.apply(program, i, limit))
					.unwrap_or_default();

				i -= removed;

				if changed {
					progress = true;
				} else if self.0.should_run(&window) {
					warn!(
						"pass {:?}::should_run was true but didn't make changes",
						self.0
					);
					tracing::debug!("{window:?}");
				}
			}

			i += 1;
		}

		progress
	}
}
