#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod change;
mod metadata;
mod pass;
pub mod passes;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	mem,
};

use tracing::{debug, info, warn};
use vmm_ir::Instruction;
use vmm_program::Program;

#[allow(clippy::wildcard_imports)]
use self::passes::*;
pub use self::{change::*, metadata::*, pass::*};

pub struct Optimizer<S: MetadataStore = HashMetadataStore> {
	program: Program,
	store: S,
}

impl<S: MetadataStore> Optimizer<S> {
	pub const fn new(program: Program, store: S) -> Self {
		Self { program, store }
	}

	pub fn optimize(&mut self) -> Result<Program, OptimizerError> {
		if self.program.is_finalized() {
			return Ok(Program::Finalized(
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

		if !matches!(counter, 1) {
			let (first_program, last_program) = (
				self.store.get_program_snapshot(1)?,
				self.store.get_program_snapshot(counter)?,
			);

			if let Some((first_program, last_program)) = Option::zip(first_program, last_program) {
				if first_program.into_iter().collect::<RawProgram>()
					!= last_program.into_iter().collect::<RawProgram>()
				{
					warn!("program instructions do not match, semantics may be different");
				}
			}
		}

		Ok(Program::Finalized(
			mem::take(&mut self.program)
				.iter()
				.cloned()
				.collect::<Vec<_>>()
				.into_boxed_slice(),
		))
	}

	fn optimize_inner(&mut self, iteration: usize) -> Result<bool, OptimizerError> {
		let starting_instruction_count = self.program.rough_estimate();

		let mut progress = false;

		self.store
			.insert_program_snapshot(iteration, &self.program)?;

		self.run_simple_passes(&mut progress);

		self.run_loop_passes(&mut progress);

		self.run_dead_code_cleanup_passes(&mut progress);

		info!(
			"Optimization iterator {iteration}: {starting_instruction_count} -> {}",
			self.program.rough_estimate()
		);

		Ok(progress)
	}

	fn run_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + Pass,
	{
		let mut pass = P::default();

		debug!("running pass {pass:?}");
		run_pass(&mut pass, self.program.as_raw(), progress);
	}

	fn run_dead_code_cleanup_passes(&mut self, progress: &mut bool) {
		self.run_pass::<RemoveRedundantMovesPass>(progress);
		self.run_pass::<RemoveRedundantWritesPass>(progress);
		self.run_pass::<RemoveEmptyLoopsPass>(progress);
		self.run_pass::<RemoveUnreachableLoopsPass>(progress);
		self.run_pass::<CleanUpStartPass>(progress);
	}

	fn run_loop_passes(&mut self, progress: &mut bool) {
		self.run_pass::<ClearCellPass>(progress);
		self.run_pass::<ClearLoopPass>(progress);
		self.run_pass::<FindZeroPass>(progress);
		self.run_pass::<MoveValuePass>(progress);
		self.run_pass::<UnrollConstantLoopsPass>(progress);
		self.run_pass::<UnrollIncrementLoopsPass>(progress);
	}

	fn run_simple_passes(&mut self, progress: &mut bool) {
		self.run_pass::<CollapseStackedInstrPass>(progress);
		self.run_pass::<SetUntouchedCellsPass>(progress);
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {
	MetadataStore(MetadataStoreError),
}

impl Display for OptimizerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::MetadataStore(e) => Display::fmt(&e, f)?,
		}

		Ok(())
	}
}

impl StdError for OptimizerError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::MetadataStore(e) => Some(e),
		}
	}
}

impl From<MetadataStoreError> for OptimizerError {
	fn from(value: MetadataStoreError) -> Self {
		Self::MetadataStore(value)
	}
}

fn run_pass<P: Pass>(pass: &mut P, v: &mut Vec<Instruction>, progress: &mut bool) {
	*progress |= pass.run_pass(v);

	if pass.should_run_on_loop() {
		for instr in v {
			if let Instruction::RawLoop(i) = instr {
				run_pass(pass, i, progress);
			}
		}
	}
}
