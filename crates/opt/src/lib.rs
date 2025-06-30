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

use tap::prelude::*;
use tracing::{debug, info, warn};
use vmm_ir::{BlockInstruction, Instruction};
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

	#[tracing::instrument("optimize program", skip(self))]
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

		let mut iteration = 1;

		let mut progress = self.optimization_pass(iteration)?;

		while progress {
			iteration += 1;
			progress = self.optimization_pass(iteration)?;
		}

		if !matches!(iteration, 1) {
			let (first_program, last_program) = (
				self.store.get_program_snapshot(1)?,
				self.store.get_program_snapshot(iteration)?,
			);

			if let Some((first_program, last_program)) = Option::zip(first_program, last_program)
				&& first_program.into_iter().collect::<RawProgram>()
					!= last_program.into_iter().collect::<RawProgram>()
			{
				warn!("program instructions do not match, semantics may be different");
			}
		}

		if let Some(program) = self.store.get_program_snapshot(iteration)? {
			self.store.insert_program_snapshot(0, &program)?;
		}

		Ok(Program::Finalized(
			mem::take(&mut self.program)
				.iter()
				.cloned()
				.collect::<Vec<_>>()
				.into_boxed_slice(),
		))
	}

	#[tracing::instrument("run pass", skip(self))]
	fn optimization_pass(&mut self, iteration: usize) -> Result<bool, OptimizerError> {
		let starting_instruction_count = self.program.rough_estimate();
		let raw_starting_instruction_count = self.program.len();

		let mut progress = false;

		self.store
			.insert_program_snapshot(iteration, &self.program)?;

		self.run_all_passes(&mut progress);

		info!(
			"{starting_instruction_count} ({raw_starting_instruction_count}) -> {} ({})",
			self.program.rough_estimate(),
			self.program.len()
		);

		Ok(progress)
	}

	fn run_pass<P>(&mut self, pass: &mut P, progress: &mut bool)
	where
		P: Debug + Pass,
	{
		pass.tap(|pass| debug!("running pass {pass:?}"))
			.pipe(|pass| run_pass(pass, self.program.as_raw(), progress));
	}

	fn run_default_peephole_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + PeepholePass,
	{
		let mut pass = PeepholeRunner::<P>::default();

		self.run_pass(&mut pass, progress);
	}

	fn run_default_range_peephole_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + RangePeepholePass,
	{
		let mut pass = RangePeepholeRunner::<P>::default();

		self.run_pass(&mut pass, progress);
	}

	fn run_default_block_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + LoopPass,
	{
		self.run_default_dynamic_loop_pass::<P>(progress);
		self.run_default_if_nz_pass::<P>(progress);
	}

	fn run_default_dynamic_loop_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + LoopPass,
	{
		self.run_default_peephole_pass::<DynamicLoopRunner<P>>(progress);
	}

	fn run_default_if_nz_pass<P>(&mut self, progress: &mut bool)
	where
		P: Debug + Default + LoopPass,
	{
		self.run_default_peephole_pass::<IfNzRunner<P>>(progress);
	}

	fn run_all_passes(&mut self, progress: &mut bool) {
		self.run_default_peephole_pass::<CollapseStackedInstrPass>(progress);
		self.run_default_peephole_pass::<CollapseRelativeInstrPass>(progress);

		self.run_default_range_peephole_pass::<InsertKnownValueHintPass>(progress);

		self.run_default_block_pass::<OptimizeClearCellPass>(progress);
		self.run_default_dynamic_loop_pass::<OptimizeClearLoopPass>(progress);
		self.run_default_block_pass::<OptimizeFindZeroPass>(progress);
		self.run_default_peephole_pass::<OptimizeSetZeroPass>(progress);
		self.run_default_block_pass::<OptimizeScaleAndMoveValPass>(progress);
		self.run_default_peephole_pass::<OptimizeFetchAndScaleValPass>(progress);
		self.run_default_peephole_pass::<OptimizeScaleValPass>(progress);
		self.run_default_peephole_pass::<OptimizeZeroedCellIncValPass>(progress);
		self.run_default_peephole_pass::<OptimizeScaleAndTakeValPass>(progress);
		self.run_default_peephole_pass::<OptimizeScaleAndSetValPass>(progress);
		self.run_default_peephole_pass::<OptimizeSetScaleValPass>(progress);
		self.run_default_peephole_pass::<OptimizeTakeValPass>(progress);
		self.run_default_peephole_pass::<OptimizeTakeFetchValPass>(progress);
		self.run_default_block_pass::<OptimizeIfNzPass>(progress);
		self.run_default_dynamic_loop_pass::<OptimizeSubCellPass>(progress);
		self.run_default_peephole_pass::<OptimizeConstantSubPass>(progress);
		self.run_default_peephole_pass::<OptimizeFetchValPass>(progress);
		self.run_default_dynamic_loop_pass::<OptimizeSetUntilZeroPass>(progress);
		self.run_default_peephole_pass::<OptimizeReplaceValPass>(progress);
		self.run_default_peephole_pass::<OptimizeFindCellByZeroPass>(progress);
		self.run_default_dynamic_loop_pass::<OptimizeShiftValsPass>(progress);
		self.run_default_peephole_pass::<OptimizeSuperInstrPass>(progress);
		self.run_default_peephole_pass::<OptimizeKnownValueHintPass>(progress);

		self.run_default_peephole_pass::<ReorderMoveChangePass>(progress);
		self.run_default_peephole_pass::<ReorderOffsetBetweenMovesPass>(progress);
		self.run_default_peephole_pass::<CombineMoveChangePass>(progress);
		self.run_default_peephole_pass::<SortIncInstrPass>(progress);
		self.run_default_peephole_pass::<SortSetInstrPass>(progress);

		self.run_default_peephole_pass::<RemoveRedundantChangeValBasicPass>(progress);
		self.run_default_peephole_pass::<RemoveRedundantChangeValOffsetPass>(progress);
		self.run_default_peephole_pass::<RemovePointlessInstrPass>(progress);
		self.run_default_peephole_pass::<RemoveRedundantScaleValInstrBasicPass>(progress);
		self.run_default_peephole_pass::<RemoveRedundantShiftsPass>(progress);
		self.run_default_peephole_pass::<RemoveRedundantCompilerHintsPass>(progress);

		self.run_default_dynamic_loop_pass::<RemoveEmptyLoopsPass>(progress);
		self.run_default_peephole_pass::<RemoveUnreachableLoopsPass>(progress);
		self.run_default_peephole_pass::<RemoveUnusedBoundaryInstrPass>(progress);
		self.run_default_dynamic_loop_pass::<RemoveInfiniteLoopsPass>(progress);

		self.run_default_peephole_pass::<UnrollConstantLoopsPass>(progress);
		self.run_default_peephole_pass::<UnrollIncrementLoopsPass>(progress);
		self.run_default_peephole_pass::<UnrollScaleAndPass>(progress);
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizerError {
	MetadataStore(MetadataStoreError),
}

impl Display for OptimizerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::MetadataStore(_) => f.write_str("issue storing metadata")?,
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

#[allow(clippy::needless_for_each)]
fn run_pass<P>(pass: &mut P, v: &mut Vec<Instruction>, progress: &mut bool)
where
	P: Pass,
{
	*progress |= pass.run_pass(v);

	if pass.should_run_on_dyn_loop() {
		for i in v.iter_mut() {
			if let Instruction::Block(BlockInstruction::DynamicLoop(i)) = i {
				let mut v = i.to_vec();

				run_pass(pass, &mut v, progress);

				*i = v.into_iter().collect();
			}
		}
	}

	if pass.should_run_on_if() {
		for i in v {
			if let Instruction::Block(BlockInstruction::IfNz(i)) = i {
				let mut v = i.to_vec();

				run_pass(pass, &mut v, progress);

				*i = v.into_iter().collect();
			}
		}
	}
}
