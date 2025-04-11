use itertools::Itertools;
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
	compile_graph::{Annotations, CompileGraph, CompileLink, CompileNode, LinkType, NodeType},
};

pub struct AnalogRepeaters;

impl<W: World> Pass<W> for AnalogRepeaters {
	fn status_message(&self) -> &'static str {
		"Combining analog repeaters"
	}

	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		'next: for i in 0..graph.node_bound() {
			let start_idx = NodeIndex::new(i);
			if !graph.contains_node(start_idx) {
				continue;
			}

			if !matches!(graph[start_idx].ty, NodeType::Comparator { .. }) {
				continue 'next;
			}

			let repeaters = graph
				.neighbors_directed(start_idx, Direction::Outgoing)
				.collect::<Vec<_>>();

			if !matches!(repeaters.len(), 15) {
				continue 'next;
			}

			if !repeaters.iter().all(|&idx| {
				graph[idx].is_removable()
					&& matches!(
						graph[idx].ty,
						NodeType::Repeater {
							delay: 1,
							facing_diode: false
						}
					)
			}) {
				continue 'next;
			}

			let Ok(end_idx) = graph
				.neighbors_directed(repeaters[0], Direction::Outgoing)
				.exactly_one()
			else {
				continue 'next;
			};

			if !matches!(graph[end_idx].ty, NodeType::Comparator { .. }) {
				continue 'next;
			}

			let mut incoming = [false; 15];
			let mut outgoing = [false; 15];
			for &repeater in &repeaters {
				let Ok(inc) = graph
					.edges_directed(repeater, Direction::Incoming)
					.exactly_one()
				else {
					continue 'next;
				};

				let Ok(out) = graph
					.edges_directed(repeater, Direction::Outgoing)
					.exactly_one()
				else {
					continue 'next;
				};

				if end_idx != out.target() {
					continue 'next;
				}

				if !matches!(inc.weight().ty, LinkType::Default) {
					continue 'next;
				}

				if !matches!(
					inc.weight().signal_strength + out.weight().signal_strength,
					14
				) {
					continue 'next;
				}

				incoming[inc.weight().signal_strength as usize] = true;
				outgoing[out.weight().signal_strength as usize] = true;
			}

			if incoming.into_iter().any(|x| !x) || outgoing.into_iter().any(|x| !x) {
				continue 'next;
			}

			for &idx in &repeaters {
				graph.remove_node(idx);
			}

			let state = graph[start_idx].state;
			let new_comparator = graph.add_node(CompileNode {
				ty: NodeType::Comparator {
					mode: ComparatorMode::Compare,
					far_input: None,
					facing_diode: false,
				},
				block: None,
				state,
				is_input: false,
				is_output: false,
				annotations: Annotations::default(),
			});

			graph.add_edge(start_idx, new_comparator, CompileLink::default(0));
			graph.add_edge(new_comparator, end_idx, CompileLink::default(0));
		}
	}
}
