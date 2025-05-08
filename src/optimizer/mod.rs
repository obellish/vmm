mod analysis;
mod change;
mod io;
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

pub use self::{analysis::*, change::*, io::*, pass::*};
#[allow(clippy::wildcard_imports)]
use crate::{Instruction, Program, passes::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Optimizer<O: OptStore = NoOpStore> {
	program: Program,
	tape_analysis_results: Vec<Box<[CellState]>>,
	output: O,
}

impl Optimizer<NoOpStore> {
	#[must_use]
	pub const fn new(program: Program) -> Self {
		Self::new_with(program, NoOpStore)
	}
}

impl<O: OptStore> Optimizer<O> {
	#[must_use]
	pub const fn new_with(program: Program, output: O) -> Self {
		Self {
			program,
			tape_analysis_results: Vec::new(),
			output,
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

		let mut progress = self.optimize_inner(counter)?;

		while progress {
			counter += 1;
			progress = self.optimize_inner(counter)?;
		}

		Ok(Program::Optimized(
			mem::take(&mut self.program)
				.iter()
				.cloned()
				.collect::<Vec<_>>()
				.into_boxed_slice(),
		))
	}

	fn optimize_inner(&mut self, iteration: usize) -> Result<bool, OptimizerError> {
		let starting_instruction_count = self.program.len();

		let mut progress = false;

		let latest_output = {
			let mut analyzer = StaticAnalyzer::new(&self.program);

			analyzer.analyze();

			analyzer.cells()
		};

		self.output
			.write_analysis_output(iteration, &latest_output)?;

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

		Ok(progress || starting_instruction_count > self.program.len())
	}

	fn run_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + Pass,
	{
		let mut pass = P::default();

		debug!("running pass {pass:?}");
		run_pass_on_vec(&mut pass, self.program.as_raw(), progress);
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {
	OptStore(OptStoreError),
}

impl Display for OptimizerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::OptStore(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for OptimizerError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::OptStore(e) => Some(e),
		}
	}
}

impl From<OptStoreError> for OptimizerError {
	fn from(value: OptStoreError) -> Self {
		Self::OptStore(value)
	}
}

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
