use std::fs;

use petgraph::{Direction, stable_graph::NodeIndex, visit::EdgeRef};
use rustc_hash::{FxBuildHasher, FxHashMap};
use vmm_blocks::blocks::ComparatorMode as CComparatorMode;
use vmm_redpiler_graph::{
	BlockPos, ComparatorMode, Link, LinkType, Node, NodeState, NodeType, serialize,
};
use vmm_world::World;

use super::Pass;
use crate::{
	CompilerInput, CompilerOptions,
	compile_graph::{CompileGraph, LinkType as CLinkType, NodeType as CNodeType},
};

pub struct ExportGraph;

impl<W: World> Pass<W> for ExportGraph {
	fn status_message(&self) -> &'static str {
		"Exporting graph"
	}

	fn should_run(&self, options: CompilerOptions) -> bool {
		options.export
	}

	fn run_pass(&self, graph: &mut CompileGraph, _: CompilerOptions, _: &CompilerInput<'_, W>) {
		let mut nodes_map = FxHashMap::with_capacity_and_hasher(graph.node_count(), FxBuildHasher);
		for node in graph.node_indices() {
			nodes_map.insert(node, nodes_map.len());
		}

		let nodes = graph
			.node_indices()
			.map(|idx| convert_node(graph, idx, &nodes_map))
			.collect::<Vec<_>>();

		fs::write("redpiler_graph.bc", serialize(nodes.as_slice()).unwrap()).unwrap();
	}
}

fn convert_node(
	graph: &CompileGraph,
	node_idx: NodeIndex,
	nodes_map: &FxHashMap<NodeIndex, usize>,
) -> Node {
	let node = &graph[node_idx];

	let mut inputs = Vec::new();
	for edge in graph.edges_directed(node_idx, Direction::Incoming) {
		let idx = nodes_map[&edge.source()];
		let weight = edge.weight();
		inputs.push(Link {
			ty: match weight.ty {
				CLinkType::Default => LinkType::Default,
				CLinkType::Side => LinkType::Side,
			},
			weight: weight.signal_strength,
			to: idx,
		});
	}

	let updates = graph
		.neighbors_directed(node_idx, Direction::Outgoing)
		.map(|idx| nodes_map[&idx])
		.collect();

	let facing_diode = match node.ty {
		CNodeType::Repeater { facing_diode, .. } | CNodeType::Comparator { facing_diode, .. } => {
			facing_diode
		}
		_ => false,
	};

	let comparator_far_input = match node.ty {
		CNodeType::Comparator { far_input, .. } => far_input,
		_ => None,
	};

	Node {
		ty: match node.ty {
			CNodeType::Repeater { delay, .. } => NodeType::Repeater(delay),
			CNodeType::Torch => NodeType::Torch,
			CNodeType::Comparator { mode, .. } => NodeType::Comparator(match mode {
				CComparatorMode::Compare => ComparatorMode::Compare,
				CComparatorMode::Subtract => ComparatorMode::Subtract,
			}),
			CNodeType::Lamp => NodeType::Lamp,
			CNodeType::Button => NodeType::Button,
			CNodeType::Lever => NodeType::Lever,
			CNodeType::PressurePlate => NodeType::PressurePlate,
			CNodeType::Trapdoor => NodeType::Trapdoor,
			CNodeType::Wire => NodeType::Wire,
			CNodeType::Constant => NodeType::Constant,
			CNodeType::NoteBlock { .. } => NodeType::NoteBlock,
		},
		blocks: node.block.map(|(pos, id)| {
			(
				BlockPos {
					x: pos.x,
					y: pos.y,
					z: pos.z,
				},
				id,
			)
		}),
		state: NodeState {
			powered: node.state.powered,
			repeated_locked: node.state.repeater_locked,
			output_strength: node.state.output_strength,
		},
		comparator_far_input,
		facing_diode,
		inputs,
		updates,
	}
}
