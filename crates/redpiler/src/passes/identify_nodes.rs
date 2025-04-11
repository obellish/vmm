use itertools::Itertools as _;
use petgraph::stable_graph::NodeIndex;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::Value;
use tracing::warn;
use vmm_blocks::{
	BlockDirection, BlockFace, BlockPos,
	blocks::{Block, entities::BlockEntity},
};
use vmm_redstone::{self, comparator, noteblock, wire};
use vmm_world::{World, for_each_block_optimized};

use super::Pass;
use crate::{
	CompilerInput, CompilerOptions,
	compile_graph::{Annotations, CompileGraph, CompileNode, NodeState, NodeType},
};

pub struct IdentifyNodes;

impl<W: World> Pass<W> for IdentifyNodes {
	fn status_message(&self) -> &'static str {
		"Identifying nodes"
	}

	fn should_run(&self, _: CompilerOptions) -> bool {
		true
	}

	fn run_pass(
		&self,
		graph: &mut CompileGraph,
		options: CompilerOptions,
		input: &CompilerInput<'_, W>,
	) {
		let ignore_wires = options.optimize;
		let plot = input.world;

		let mut first_pass = FxHashMap::default();
		let mut second_pass = FxHashSet::default();

		let (first_pos, second_pos) = input.bounds;

		for_each_block_optimized(plot, first_pos, second_pos, |pos| {
			for_pos(
				graph,
				&mut first_pass,
				&mut second_pass,
				ignore_wires,
				options.wire_dot_out,
				plot,
				pos,
			);
		});

		for pos in second_pass {
			apply_annotations(graph, options, &first_pass, plot, pos);
		}
	}
}

pub enum NodeAnnotation {}

impl NodeAnnotation {
	fn parse(s: &str) -> Option<Self> {
		let s = s.trim().to_ascii_lowercase();
		if !(s.starts_with('[') && s.ends_with(']')) {
			return None;
		}

		None
	}

	const fn apply(
		self,
		_graph: &mut CompileGraph,
		_node_idx: NodeIndex,
		_options: CompilerOptions,
	) -> Result<(), String> {
		match self {}
	}
}

fn for_pos<W: World>(
	graph: &mut CompileGraph,
	first_pass: &mut FxHashMap<BlockPos, NodeIndex>,
	second_pass: &mut FxHashSet<BlockPos>,
	ignore_wires: bool,
	wire_dot_out: bool,
	world: &W,
	pos: BlockPos,
) {
	let id = world.get_block_raw(pos);
	let block = Block::from_id(id);

	if matches!(block, Block::Sign { .. } | Block::WallSign { .. }) {
		second_pass.insert(pos);
		return;
	}

	let Some((ty, state)) = identify_block(block, pos, world) else {
		return;
	};

	let is_input = matches!(
		ty,
		NodeType::Button | NodeType::Lever | NodeType::PressurePlate
	);

	let is_output = matches!(
		ty,
		NodeType::Trapdoor | NodeType::Lamp | NodeType::NoteBlock { .. }
	) || matches!(block, Block::RedstoneWire { wire } if wire_dot_out && wire::is_dot(wire));

	if ignore_wires && matches!(ty, NodeType::Wire) && !(is_input | is_output) {
		return;
	}

	let node_idx = graph.add_node(CompileNode {
		ty,
		block: Some((pos, id)),
		state,
		is_input,
		is_output,
		annotations: Annotations::default(),
	});

	first_pass.insert(pos, node_idx);
}

fn identify_block<W: World>(
	block: Block,
	pos: BlockPos,
	world: &W,
) -> Option<(NodeType, NodeState)> {
	let (ty, state) = match block {
		Block::RedstoneRepeater { repeater } => (
			NodeType::Repeater {
				delay: repeater.delay,
				facing_diode: vmm_redstone::is_diode(
					world.get_block(pos.offset((!repeater.facing).into())),
				),
			},
			NodeState::repeater(repeater.powered, repeater.locked),
		),
		Block::RedstoneComparator { comparator } => (
			NodeType::Comparator {
				mode: comparator.mode,
				far_input: comparator::get_far_input(world, pos, comparator.facing),
				facing_diode: vmm_redstone::is_diode(
					world.get_block(pos.offset((!comparator.facing).into())),
				),
			},
			NodeState::comparator(
				comparator.powered,
				if let Some(BlockEntity::Comparator { output_strength }) =
					world.get_block_entity(pos)
				{
					*output_strength
				} else {
					0
				},
			),
		),
		Block::RedstoneTorch { lit } | Block::RedstoneWallTorch { lit, .. } => {
			(NodeType::Torch, NodeState::simple(lit))
		}
		Block::RedstoneWire { wire } => (NodeType::Wire, NodeState::signal_strength(wire.power)),
		Block::StoneButton { button } => (NodeType::Button, NodeState::simple(button.powered)),
		Block::RedstoneLamp { lit } => (NodeType::Lamp, NodeState::simple(lit)),
		Block::Lever { lever } => (NodeType::Lever, NodeState::simple(lever.powered)),
		Block::StonePressurePlate { powered } => {
			(NodeType::PressurePlate, NodeState::simple(powered))
		}
		Block::IronTrapdoor { powered, .. } => (NodeType::Trapdoor, NodeState::simple(powered)),
		Block::RedstoneBlock {} => (NodeType::Constant, NodeState::signal_strength(15)),
		Block::NoteBlock { note, powered, .. } if noteblock::is_noteblock_unblocked(world, pos) => {
			let instrument = noteblock::get_noteblock_instrument(world, pos);
			(
				NodeType::NoteBlock { instrument, note },
				NodeState::simple(powered),
			)
		}
		block if comparator::has_override(block) => (
			NodeType::Constant,
			NodeState::signal_strength(comparator::get_override(block, world, pos)),
		),
		_ => return None,
	};

	Some((ty, state))
}

fn apply_annotations<W: World>(
	graph: &mut CompileGraph,
	options: CompilerOptions,
	first_pass: &FxHashMap<BlockPos, NodeIndex>,
	world: &W,
	pos: BlockPos,
) {
	let block = world.get_block(pos);
	let annotations = parse_sign_annotations(world.get_block_entity(pos));
	if annotations.is_empty() {
		return;
	}

	let targets = match block {
		Block::Sign { rotation, .. } => {
			if let Some(facing) = BlockDirection::from_rotation(rotation) {
				let behind = pos.offset((!facing).into());
				[behind].to_vec()
			} else {
				warn!("found sign with annotations, but bad rotation as {pos}");
				return;
			}
		}
		Block::WallSign { facing, .. } => {
			let behind = pos.offset((!facing).into());
			[
				behind,
				behind.offset(BlockFace::Top),
				behind.offset(BlockFace::Bottom),
			]
			.to_vec()
		}
		_ => panic!("block unimplemented for second pass"),
	};

	let target = targets.iter().find_map(|pos| first_pass.get(pos));
	if let Some(&node_idx) = target {
		for annotation in annotations {
			let result = annotation.apply(graph, node_idx, options);
			if let Err(msg) = result {
				warn!("{msg} at {pos}");
			}
		}
	} else {
		warn!("could not find component for annotation at {pos}");
	}
}

fn parse_sign_annotations(entity: Option<&BlockEntity>) -> Vec<NodeAnnotation> {
	if let Some(BlockEntity::Sign(sign)) = entity {
		sign.front_rows
			.iter()
			.flat_map(|row| serde_json::from_str(row))
			.filter_map(|json: Value| {
				NodeAnnotation::parse(json.as_object()?.get("text")?.as_str()?)
			})
			.collect()
	} else {
		Vec::new()
	}
}
