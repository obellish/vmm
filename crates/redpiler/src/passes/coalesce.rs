use itertools::Itertools;
use petgraph::{
	graph::{EdgeIndex, WalkNeighbors}, stable_graph::NodeIndex, visit::{EdgeRef, NodeIndexable}, Direction
};
use tracing::trace;
use vmm_world::World;

use super::Pass;
use crate::{
	CompilerInput, CompilerOptions,
	compile_graph::{CompileGraph, LinkType, NodeType},
};

pub struct Coalesce;

impl<W: World> Pass<W> for Coalesce {
	fn status_message(&self) -> &'static str {
		"Combining duplicate logic"
	}

	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		loop {
			let num_coalesced = run_iteration(graph);
			trace!("iteration combined {num_coalesced} nodes");
			if matches!(num_coalesced, 0) {
				break;
			}
		}
	}
}

fn run_iteration(graph: &mut CompileGraph) -> usize {
	let mut num_coalesced = 0;

	for idx in (0..graph.node_bound()).map(NodeIndex::new) {
		if !graph.contains_node(idx) {
			continue;
		}

		let node = &graph[idx];

		if matches!(node.ty, NodeType::Comparator { .. }) || !node.is_removable() {
			continue;
		}

		let Ok(edge) = graph.edges_directed(idx, Direction::Incoming).exactly_one() else {
			continue;
		};

		if !matches!(edge.weight().ty, LinkType::Default) {
			continue;
		}

		let source = edge.source();
		if matches!(graph[source].ty, NodeType::Comparator { .. }) {
			continue;
		}

		num_coalesced += coalesce_outgoing(graph, source, idx);
	}

	num_coalesced
}

fn coalesce(graph: &mut CompileGraph, node: NodeIndex, into: NodeIndex) {
	let mut walk_outgoing = graph.neighbors_directed(node, Direction::Outgoing).detach();
	while let Some(edge_idx) = walk_outgoing.next_edge(graph) {
		let dest = graph.edge_endpoints(edge_idx).unwrap().1;
		let weight = graph.remove_edge(edge_idx).unwrap();
		graph.add_edge(into, dest, weight);
	}

	graph.remove_node(node);
}

fn coalesce_outgoing(
	graph: &mut CompileGraph,
	source_idx: NodeIndex,
	into_idx: NodeIndex,
) -> usize {
	let mut num_coalesced = 0;
	let mut walk_outgoing = graph
		.neighbors_directed(source_idx, Direction::Outgoing)
		.detach();
	while let Some(edge_idx) = walk_outgoing.next_edge(graph) {
		let dest_idx = graph.edge_endpoints(edge_idx).unwrap().1;
		if dest_idx == into_idx {
			continue;
		}

		let dest = &graph[dest_idx];
		let into = &graph[into_idx];

		if dest.ty == into.ty
			&& dest.is_removable()
			&& matches!(
				graph
					.neighbors_directed(dest_idx, Direction::Incoming)
					.count(),
				1
			) {
			coalesce(graph, dest_idx, into_idx);
			num_coalesced += 1;
		}
	}

	num_coalesced
}
