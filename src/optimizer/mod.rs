mod analysis;
mod change;
mod pass;
#[cfg(test)]
mod tests;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	mem,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

pub use self::{analysis::*, change::*, pass::*};
#[allow(clippy::wildcard_imports)]
use crate::{Instruction, Program, passes::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Optimizer {
	program: Program,
}

impl Optimizer {
	#[must_use]
	pub const fn new(current_unit: Program) -> Self {
		Self {
			program: current_unit,
		}
	}

	pub fn optimize(&mut self) -> Result<Program, OptimizerError> {
		if self.program.is_optimized() {
			return Ok(Program::Optimized(
				mem::take(&mut self.program)
					.iter()
					.cloned()
					.collect::<Vec<_>>()
					.into_boxed_slice(),
			));
		}

		let mut counter = 1;

		let mut progress = self.optimize_inner(counter);

		while progress {
			counter += 1;
			progress = self.optimize_inner(counter);
		}

		Ok(Program::Optimized(
			mem::take(&mut self.program)
				.iter()
				.cloned()
				.collect::<Vec<_>>()
				.into_boxed_slice(),
		))
	}

	fn optimize_inner(&mut self, iteration: usize) -> bool {
		let starting_instruction_count = self.program.len();

		let mut progress = false;

		self.run_pass::<CombineIncInstrPass>(&mut progress);
		self.run_pass::<CombineMoveInstrPass>(&mut progress);
		self.run_pass::<CombineSetInstrPass>(&mut progress);

		self.run_pass::<SetZeroPass>(&mut progress);
		self.run_pass::<FindZeroPass>(&mut progress);
		self.run_pass::<SetUntouchedCellsPass>(&mut progress);

		self.run_pass::<MoveValuePass>(&mut progress);
		self.run_pass::<UnrollConstantLoopsPass>(&mut progress);

		self.run_pass::<RemoveEmptyLoopsPass>(&mut progress);
		self.run_pass::<RemoveRedundantWritesPass>(&mut progress);

		info!(
			"Optimization iteration {iteration}: {starting_instruction_count} -> {}",
			self.program.len()
		);

		progress || starting_instruction_count > self.program.len()
	}

	#[tracing::instrument(skip(self))]
	fn run_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + Pass,
	{
		let mut pass = P::default();

		debug!("running pass");
		run_pass_on_vec(&mut pass, self.program.as_raw(), progress);
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {}

impl Display for OptimizerError {
	fn fmt(&self, _f: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for OptimizerError {}

fn run_pass_on_vec<P: Pass>(pass: &mut P, v: &mut Vec<Instruction>, progress: &mut bool) {
	*progress |= pass.run_pass(v);

	if pass.should_run_on_loop() {
		for instr in v {
			if let Instruction::RawLoop(i) = instr {
				run_pass_on_vec(pass, i, progress);
			}
		}
	}
}
