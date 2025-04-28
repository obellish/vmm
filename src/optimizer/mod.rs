mod change;
mod options;
mod pass;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	mem,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

pub use self::{change::*, options::*, pass::*};
#[allow(clippy::wildcard_imports)]
use crate::{ExecutionUnit, Program, passes::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Optimizer {
	current_unit: ExecutionUnit,
	options: OptimizerOptions,
}

impl Optimizer {
	#[must_use]
	pub const fn new(current_unit: ExecutionUnit, options: OptimizerOptions) -> Self {
		Self {
			current_unit,
			options,
		}
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
			self.current_unit.tape().clone(),
		))
	}

	fn optimize_inner(&mut self, iteration: usize) -> bool {
		let starting_instruction_count = self.current_unit.program().len();

		let passes = self.resolve_and_run_passes();

		if self.options.verbose {
			info!(
				"Optimization iteration {iteration}: {starting_instruction_count} -> {}",
				self.current_unit.program().len()
			);
		}

		let mut progress =false;

		for pass in passes {
			debug!("running pass {}", pass.name());
			progress |= pass.run_pass(&mut self.current_unit);
		}

		progress || starting_instruction_count > self.current_unit.program().len()
	}

	fn resolve_and_run_passes(&self) -> Vec<Box<dyn Pass>> {
		let mut passes = Vec::new();

		if self.options.combine_instructions {
			let b: Box<dyn Pass> = Box::new(CombineInstrPass);
			passes.push(b);
		}

		passes
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {}

impl Display for OptimizerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for OptimizerError {}
