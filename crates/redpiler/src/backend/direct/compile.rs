use std::sync::Arc;

use itertools::Itertools;
use petgraph::{Direction, stable_graph::NodeIndex, visit::EdgeRef};
use rustc_hash::{FxBuildHasher, FxHashMap};
use tracing::trace;
use vmm_blocks::{
	BlockPos,
	blocks::{Block, Instrument},
};
use vmm_world::TickEntry;

use super::{
	DirectBackend,
	node::{ForwardLink, Node, NodeId, NodeInput, NodeType, Nodes, NonMaxU8},
};
use crate::{
	CompilerOptions, TaskMonitor,
	compile_graph::{CompileGraph, LinkType, NodeType as CNodeType},
};

#[derive(Debug, Default)]
struct FinalGraphStats {
	update_link_count: usize,
	side_link_count: usize,
	default_link_count: usize,
	nodes_bytes: usize,
}

pub fn compile(
	backend: &mut DirectBackend,
	graph: CompileGraph,
	ticks: Vec<TickEntry>,
	options: CompilerOptions,
	_monitor: Arc<TaskMonitor>,
) {
	let mut nodes_map = FxHashMap::with_capacity_and_hasher(graph.node_count(), FxBuildHasher);
	for node in graph.node_indices() {
		nodes_map.insert(node, nodes_map.len());
	}
	let nodes_len = nodes_map.len();

	let mut stats = FinalGraphStats::default();
	let nodes = graph
		.node_indices()
		.map(|idx| {
			compile_node(
				&graph,
				idx,
				nodes_len,
				&nodes_map,
				&mut backend.noteblock_info,
				&mut stats,
			)
		})
		.collect::<Vec<_>>();
	stats.nodes_bytes = nodes_len * std::mem::size_of::<Node>();
	trace!("{stats:#?}");

	backend.blocks = graph
		.node_weights()
		.map(|node| node.block.map(|(pos, id)| (pos, Block::from_id(id))))
		.collect();

	backend.nodes = Nodes::new(nodes);

	for i in 0..backend.blocks.len() {
		if let Some((pos, _)) = backend.blocks[i] {
			backend.pos_map.insert(pos, backend.nodes.get(i));
		}
	}

	for entry in ticks {
		if let Some(node) = backend.pos_map.get(&entry.pos) {
			backend
				.scheduler
				.schedule_tick(*node, entry.ticks_left as usize, entry.tick_priority);
			backend.nodes[*node].pending_tick = true;
		}
	}

	if options.export_dot_graph {
		std::fs::write("backend_graph.dot", format!("{backend}")).unwrap();
	}
}

fn compile_node(
	graph: &CompileGraph,
	node_idx: NodeIndex,
	nodes_len: usize,
	nodes_map: &FxHashMap<NodeIndex, usize>,
	noteblock_info: &mut Vec<(BlockPos, Instrument, u32)>,
	stats: &mut FinalGraphStats,
) -> Node {
	const MAX_INPUTS: usize = 255;

	let node = &graph[node_idx];

	let mut default_input_count = 0;
	let mut side_input_count = 0;

	let mut default_inputs = NodeInput {
		signal_strength_counts: [0; 16],
	};
	let mut side_inputs = NodeInput {
		signal_strength_counts: [0; 16],
	};
	for edge in graph.edges_directed(node_idx, Direction::Incoming) {
		let weight = edge.weight();
		let distance = weight.signal_strength;
		let source = edge.source();
		let signal_strength = graph[source].state.output_strength.saturating_sub(distance);
		match weight.ty {
			LinkType::Default => {
				assert!(
					(default_input_count < MAX_INPUTS),
					"exceeded the maximum number of default inputs {MAX_INPUTS}"
				);
				default_input_count += 1;
				default_inputs.signal_strength_counts[signal_strength as usize] += 1;
			}
			LinkType::Side => {
				assert!(
					(side_input_count < MAX_INPUTS),
					"exceeded the maximum number of side inputs {MAX_INPUTS}"
				);

				side_input_count += 1;
				side_inputs.signal_strength_counts[signal_strength as usize] += 1;
			}
		}
	}

	stats.default_link_count = default_input_count;
	stats.side_link_count = side_input_count;

	let updates = if matches!(node.ty, CNodeType::Constant) {
		Vec::new()
	} else {
		graph
			.edges_directed(node_idx, Direction::Outgoing)
			.sorted_by_key(|edge| nodes_map[&edge.target()])
			.into_group_map_by(|edge| std::mem::discriminant(&graph[edge.target()].ty))
			.into_values()
			.flatten()
			.map(|edge| unsafe {
				let idx = edge.target();
				let idx = nodes_map[&idx];
				assert!(idx < nodes_len);

				let target_id = NodeId::from_index(idx);

				let weight = edge.weight();
				ForwardLink::new(
					target_id,
					matches!(weight.ty, LinkType::Side),
					weight.signal_strength,
				)
			})
			.collect()
	};

	stats.update_link_count += updates.len();

	let ty = match &node.ty {
		CNodeType::Repeater {
			delay,
			facing_diode,
		} => NodeType::Repeater {
			delay: *delay,
			facing_diode: *facing_diode,
		},
		CNodeType::Torch => NodeType::Torch,
		CNodeType::Comparator {
			mode,
			far_input,
			facing_diode,
		} => NodeType::Comparator {
			mode: *mode,
			far_input: far_input.and_then(NonMaxU8::new),
			facing_diode: *facing_diode,
		},
		CNodeType::Lamp => NodeType::Lamp,
		CNodeType::Button => NodeType::Button,
		CNodeType::Lever => NodeType::Lever,
		CNodeType::PressurePlate => NodeType::PressurePlate,
		CNodeType::Trapdoor => NodeType::Trapdoor,
		CNodeType::Wire => NodeType::Wire,
		CNodeType::Constant => NodeType::Constant,
		CNodeType::NoteBlock { instrument, note } => {
			let noteblock_id = noteblock_info.len().try_into().unwrap();
			noteblock_info.push((node.block.unwrap().0, *instrument, *note));
			NodeType::NoteBlock { noteblock_id }
		}
	};

	Node {
		ty,
		default_inputs,
		side_inputs,
		updates,
		powered: node.state.powered,
		output_power: node.state.output_strength,
		locked: node.state.repeater_locked,
		pending_tick: false,
		changed: false,
		is_io: node.is_input || node.is_output,
	}
}
