mod analog_repeaters;
mod clamp_weights;
mod coalesce;
mod constant_coalesce;
mod constant_fold;
mod dedup_links;
mod export_graph;
mod identify_nodes;
mod input_search;
mod prune_orphans;
mod unreachable_output;

use std::{sync::Arc, time::Instant};

use tracing::trace;
use vmm_world::World;

use super::{CompilerInput, CompilerOptions, TaskMonitor, compile_graph::CompileGraph};

pub struct PassManager<'p, W: World> {
	passes: &'p [&'p dyn Pass<W>],
}

impl<'p, W: World> PassManager<'p, W> {
	pub const fn new(passes: &'p [&dyn Pass<W>]) -> Self {
		Self { passes }
	}

	pub fn run_passes(
		&self,
		options: CompilerOptions,
		input: &CompilerInput<'_, W>,
		monitor: Arc<TaskMonitor>,
	) -> CompileGraph {
		let mut graph = CompileGraph::new();

		monitor.set_max_progress(self.passes.len() + 1);

		for &pass in self.passes {
			if !pass.should_run(options) {
				trace!("skipping pass: {}", pass.name());
				monitor.increment_progress();
				continue;
			}

			if monitor.cancelled() {
				return graph;
			}

			trace!("running pass: {}", pass.name());
			monitor.set_message(pass.status_message());
			let start = Instant::now();

			pass.run_pass(&mut graph, options, input);

			trace!("completed pass in {:?}", start.elapsed());
			trace!("node_count: {}", graph.node_count());
			trace!("edge_count: {}", graph.edge_count());
			monitor.increment_progress();
		}

		graph
	}
}

impl<W: World> Default for PassManager<'_, W> {
	fn default() -> Self {
		Self::new(&[
			&self::identify_nodes::IdentifyNodes,
			&self::input_search::InputSearch,
			&self::clamp_weights::ClampWeights,
			&self::dedup_links::DedupLinks,
			&self::analog_repeaters::AnalogRepeaters,
			&self::constant_fold::ConstantFold,
			&self::unreachable_output::UnreachableOutput,
			&self::constant_coalesce::ConstantCoalesce,
			&self::coalesce::Coalesce,
			&self::prune_orphans::PruneOrphans,
			&self::export_graph::ExportGraph,
		])
	}
}

pub trait Pass<W: World> {
	fn run_pass(
		&self,
		graph: &mut CompileGraph,
		options: CompilerOptions,
		input: &CompilerInput<'_, W>,
	);

	fn status_message(&self) -> &'static str;

	fn name(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	fn should_run(&self, options: CompilerOptions) -> bool {
		options.optimize
	}
}
