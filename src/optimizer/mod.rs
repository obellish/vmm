mod analysis;
mod change;
mod pass;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	mem,
	sync::LazyLock,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

pub use self::{analysis::*, change::*, pass::*};
#[allow(clippy::wildcard_imports)]
use crate::{ExecutionUnit, Program, passes::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Optimizer {
	current_unit: ExecutionUnit,
	current_analysis_results: Option<AnalysisResults>,
}

impl Optimizer {
	#[must_use]
	pub const fn new(current_unit: ExecutionUnit) -> Self {
		Self {
			current_unit,
			current_analysis_results: None,
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

		let res = ExecutionUnit::optimized(
			self.current_unit.program().iter().copied(),
			self.current_unit.tape().clone(),
		);

		mem::take(&mut self.current_unit);

		Ok(res)
	}

	fn optimize_inner(&mut self, iteration: usize) -> bool {
		let starting_instruction_count = self.current_unit.program().len();

		let passes: &[&dyn Pass] = &[
			&CombineZeroLoopInstrPass,
			&CombineAddInstrPass,
			&CombineMoveInstrPass,
			&RemoveEmptyLoopsPass,
			&SetUntouchedCells,
		];

		let mut progress = false;

<<<<<<< HEAD
		self.current_analysis_results = {
			debug!("running cell analysis");
			let analyzer = Analyzer::new(&self.current_unit);

			Some(analyzer.analyze())
		};

		self.run_pass(
			SetCells,
			self.current_analysis_results.clone().unwrap(),
			&mut progress,
		);
		self.run_pass(CombineZeroLoopInstrPass, (), &mut progress);
		self.run_pass(CombineAddInstrPass, (), &mut progress);
		self.run_pass(CombineMoveInstrPass, (), &mut progress);
		self.run_pass(RemoveEmptyLoopsPass, (), &mut progress);
=======
		for pass in passes {
			debug!("running pass {}", pass.name());
			progress |= pass.run_pass(&mut self.current_unit);
		}
>>>>>>> parent of fea49c6 (more tracing and mutable passes)

		info!(
			"Optimization iteration {iteration}: {starting_instruction_count} -> {}",
			self.current_unit.program().len()
		);

		progress || starting_instruction_count > self.current_unit.program().len()
	}
<<<<<<< HEAD

	#[tracing::instrument(skip(self))]
	fn run_pass<P, S: Debug>(&mut self, mut pass: P, state: S, progress: &mut bool)
	where
		P: Debug + Pass<State = S>,
	{
		debug!("running pass {}", pass.name());
		*progress |= pass.run_pass(&mut self.current_unit, state);
	}
=======
>>>>>>> parent of fea49c6 (more tracing and mutable passes)
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {}

impl Display for OptimizerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for OptimizerError {}
