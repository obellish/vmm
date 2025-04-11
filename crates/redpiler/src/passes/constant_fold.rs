use petgraph::{
	Direction,
	stable_graph::NodeIndex,
	visit::{EdgeRef, NodeIndexable},
};
use tracing::trace;
use vmm_blocks::blocks::ComparatorMode;
use vmm_world::World;

use super::Pass;
use crate::{
	CompilerInput, CompilerOptions,
	compile_graph::{CompileGraph, LinkType, NodeType},
};

pub struct ConstantFold;

impl<W: World> Pass<W> for ConstantFold {
	fn status_message(&self) -> &'static str {
		"Constant folding"
	}

	fn run_pass(
		&self,
		graph: &mut CompileGraph,
		_: CompilerOptions,
		_: &CompilerInput<'_, W>,
	) {
		loop {
			let num_folded = fold(graph);
			if matches!(num_folded, 0) {
				break;
			}

			trace!("fold iteration: {num_folded} nodes");
		}
	}
}

fn fold(graph: &mut CompileGraph) -> usize {
	let mut num_folded = 0;

	'nodes: for idx in (0..graph.node_bound()).map(NodeIndex::new) {
		if !graph.contains_node(idx) {
			continue;
		}

		let mut default_power = 0;
		let mut side_power = 0;

		for edge in graph.edges_directed(idx, Direction::Incoming) {
			let constant = &graph[edge.source()];
			if !matches!(constant.ty, NodeType::Constant) {
				continue 'nodes;
			}

			match edge.weight().ty {
				LinkType::Default => {
					default_power = default_power.max(
						constant
							.state
							.output_strength
							.saturating_sub(edge.weight().signal_strength),
					);
				}
				LinkType::Side => {
					side_power = side_power.max(
						constant
							.state
							.output_strength
							.saturating_sub(edge.weight().signal_strength),
					);
				}
			}
		}

		let new_power = match graph[idx].ty {
			NodeType::Comparator {
				mode, far_input, ..
			} => {
				if let Some(far_override) = far_input {
					if default_power < 15 {
						default_power = far_override;
					}
				}

				match mode {
					ComparatorMode::Compare => {
						if default_power >= side_power {
							default_power
						} else {
							0
						}
					}
					ComparatorMode::Subtract => default_power.saturating_sub(side_power),
				}
			}
			NodeType::Repeater { .. } => {
				if graph[idx].state.repeater_locked {
					graph[idx].state.output_strength
				} else if default_power > 0 {
					15
				} else {
					0
				}
			}
			NodeType::Torch => {
				if default_power > 0 {
					0
				} else {
					15
				}
			}
			_ => continue,
		};

		graph[idx].ty = NodeType::Constant;
		graph[idx].state.output_strength = new_power;

		let mut incoming = graph.neighbors_directed(idx, Direction::Incoming).detach();
		while let Some(edge) = incoming.next_edge(graph) {
			graph.remove_edge(edge);
		}

		num_folded += 1;
	}

	num_folded
}
