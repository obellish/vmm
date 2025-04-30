mod change;
mod pass;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	mem,
};

use serde::{Deserialize, Serialize};
use tracing::info;

pub use self::{change::*, pass::*};
#[allow(clippy::wildcard_imports)]
use crate::{ExecutionUnit, passes::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Optimizer {
	current_unit: ExecutionUnit,
}

impl Optimizer {
	#[must_use]
	pub const fn new(current_unit: ExecutionUnit) -> Self {
		Self { current_unit }
	}

	pub fn optimize(&mut self) -> Result<ExecutionUnit, OptimizerError> {
		if self.current_unit.program().is_optimized() {
			return Ok(mem::take(&mut self.current_unit));
		}

		let mut counter = 1;

		let mut progress = self.optimize_inner(counter);

		while progress {
			counter += 1;
			progress = self.optimize_inner(counter);
		}

		Ok(ExecutionUnit::optimized(
			self.current_unit.program().iter().copied(),
		))
	}

	fn optimize_inner(&mut self, iteration: usize) -> bool {
		let starting_instruction_count = self.current_unit.program().len();

		let mut progress = false;

		self.run_pass(CombineAddInstrPass, &mut progress);
		self.run_pass(CombineMoveInstrPass, &mut progress);
		self.run_pass(CombineZeroLoopInstrPass, &mut progress);
		self.run_pass(SetUntouchedCells, &mut progress);
		self.run_pass(RemoveEmptyLoopsPass, &mut progress);

		info!(
			"Optimization iteration {iteration}: {starting_instruction_count} -> {}",
			self.current_unit.program().len()
		);

		progress || starting_instruction_count > self.current_unit.program().len()
	}

	#[tracing::instrument(skip(self))]
	fn run_pass<P>(&mut self, pass: P, progress: &mut bool)
	where
		P: Debug + Pass,
	{
		*progress |= pass.run_pass(&mut self.current_unit);
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
