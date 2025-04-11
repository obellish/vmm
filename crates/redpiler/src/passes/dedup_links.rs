use petgraph::{
	Direction,
	stable_graph::NodeIndex,
	visit::{EdgeRef, NodeIndexable},
};
use vmm_world::World;

use super::Pass;
use crate::{CompilerInput, CompilerOptions, compile_graph::CompileGraph};

pub struct DedupLinks;

impl<W: World> Pass<W> for DedupLinks {
	fn status_message(&self) -> &'static str {
		"Deduplicating links"
	}

	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		for idx in (0..graph.node_bound()).map(NodeIndex::new) {
			if !graph.contains_node(idx) {
				continue;
			}

			let mut edges = graph.neighbors_directed(idx, Direction::Incoming).detach();
			while let Some(edge_idx) = edges.next_edge(graph) {
				let edge = &graph[edge_idx];
				let source_idx = graph.edge_endpoints(edge_idx).unwrap().0;

				let mut should_remove = false;
				for other_edge in graph.edges_directed(idx, Direction::Incoming) {
					if other_edge.id() != edge_idx
						&& other_edge.source() == source_idx
						&& other_edge.weight().ty == edge.ty
						&& other_edge.weight().signal_strength <= edge.signal_strength
					{
						should_remove = true;
						break;
					}
				}

				if should_remove {
					graph.remove_edge(edge_idx);
				}
			}
		}
	}
}
