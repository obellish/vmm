use itertools::Itertools as _;
use petgraph::Direction;
use rustc_hash::FxHashSet;
use vmm_world::World;

use super::Pass;
use crate::{CompilerInput, CompilerOptions, compile_graph::CompileGraph};

pub struct PruneOrphans;

impl<W: World> Pass<W> for PruneOrphans {
	fn status_message(&self) -> &'static str {
		"Pruning orphans"
	}

	fn should_run(&self, options: CompilerOptions) -> bool {
		options.io_only && options.optimize
	}

	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		let mut to_visit = graph
			.node_indices()
			.filter(|&idx| !graph[idx].is_removable())
			.collect::<Vec<_>>();

		let mut visited = FxHashSet::default();
		while let Some(idx) = to_visit.pop() {
			if visited.insert(idx) {
				to_visit.extend(graph.neighbors_directed(idx, Direction::Incoming));
			}
		}

		graph.retain_nodes(|_, idx| visited.contains(&idx));
	}
}
