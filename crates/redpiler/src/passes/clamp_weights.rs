use vmm_world::World;

use super::Pass;
use crate::{CompilerInput, CompilerOptions, compile_graph::CompileGraph};

pub struct ClampWeights;

impl<W: World> Pass<W> for ClampWeights {
	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		graph.retain_edges(|g, edge| g[edge].signal_strength < 15);
	}

	fn should_run(&self, _: CompilerOptions) -> bool {
		true
	}

	fn status_message(&self) -> &'static str {
		"Clamping weights"
	}
}
