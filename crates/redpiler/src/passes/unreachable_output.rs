use petgraph::{
	Direction,
	stable_graph::NodeIndex,
	visit::{EdgeRef, NodeIndexable},
};
use vmm_blocks::blocks::ComparatorMode;
use vmm_world::World;

use super::Pass;
use crate::{
	CompilerInput, CompilerOptions,
	compile_graph::{CompileGraph, LinkType, NodeType},
};

pub struct UnreachableOutput;

impl<W: World> Pass<W> for UnreachableOutput {
	fn status_message(&self) -> &'static str {
		"Pruning unreachable comparator outputs"
	}

	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		for idx in (0..graph.node_bound()).map(NodeIndex::new) {
			if !graph.contains_node(idx) {
				continue;
			}

			if !matches!(
				graph[idx].ty,
				NodeType::Comparator {
					mode: ComparatorMode::Subtract,
					..
				}
			) {
				continue;
			}

			let max_input = 15u8;

			let mut side_inputs = graph
				.edges_directed(idx, Direction::Incoming)
				.filter(|e| matches!(e.weight().ty, LinkType::Side));

			let Some(constant_edge) = side_inputs.next() else {
				continue;
			};

			let constant_idx = constant_edge.source();

			if side_inputs.next().is_some() {
				continue;
			}

			if !matches!(graph[constant_idx].ty, NodeType::Constant) {
				continue;
			}

			let constant = graph[constant_idx].state.output_strength;
			let max_output = max_input.saturating_sub(constant);

			let mut outgoing = graph.neighbors_directed(idx, Direction::Outgoing).detach();
			while let Some((edge_idx, _)) = outgoing.next(graph) {
				if graph[edge_idx].signal_strength >= max_output {
					graph.remove_edge(edge_idx);
				}
			}
		}
	}
}
