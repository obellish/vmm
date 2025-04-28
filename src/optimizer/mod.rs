use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	mem,
};

use serde::{Deserialize, Serialize};

use super::ExecutionUnit;
use crate::Program;

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
		if self.current_unit.has_started() {
			return Err(OptimizerError::AlreadyStarted);
		}

		if self.current_unit.program().is_optimized() {
			return Ok(mem::take(&mut self.current_unit));
		}

		Ok(ExecutionUnit::optimized(
			self.current_unit.program().iter().copied(),
			self.current_unit.tape().clone(),
		))
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {
	AlreadyStarted,
}

impl Display for OptimizerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::AlreadyStarted => f.write_str("execution unit has already started"),
		}
	}
}

impl StdError for OptimizerError {}
