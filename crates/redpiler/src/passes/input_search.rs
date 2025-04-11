use std::collections::VecDeque;

use petgraph::{stable_graph::NodeIndex, visit::NodeIndexable};
use rustc_hash::FxHashMap;
use vmm_blocks::{
	BlockDirection, BlockFace, BlockPos,
	blocks::{Block, ButtonFace, LeverFace, RedstoneRepeater},
};
use vmm_redstone::{self, comparator, wire};
use vmm_world::World;

use super::Pass;
use crate::{
	CompilerInput, CompilerOptions,
	compile_graph::{CompileGraph, CompileLink, LinkType},
};

pub struct InputSearch;

impl<W: World> Pass<W> for InputSearch {
	fn should_run(&self, _: CompilerOptions) -> bool {
		true
	}

	fn status_message(&self) -> &'static str {
		"Searching for links"
	}

	fn run_pass(
		&self,
		graph: &mut CompileGraph,
		_: CompilerOptions,
		input: &CompilerInput<'_, W>,
	) {
		let mut state = InputSearchState::new(input.world, graph);
		state.search();
	}
}

struct InputSearchState<'a, W: World> {
	world: &'a W,
	graph: &'a mut CompileGraph,
	pos_map: FxHashMap<BlockPos, NodeIndex>,
}

#[allow(clippy::unused_self)]
impl<'a, W: World> InputSearchState<'a, W> {
	fn new(world: &'a W, graph: &'a mut CompileGraph) -> Self {
		let mut pos_map = FxHashMap::default();
		for id in graph.node_indices() {
			let (pos, _) = graph[id].block.unwrap();
			pos_map.insert(pos, id);
		}

		Self {
			world,
			graph,
			pos_map,
		}
	}

	fn provides_weak_power(&self, block: Block, side: BlockFace) -> bool {
		match block {
			Block::RedstoneWallTorch { facing, .. } if BlockFace::from(facing) != side => true,
			Block::RedstoneBlock {}
			| Block::RedstoneTorch { .. }
			| Block::Lever { .. }
			| Block::StoneButton { .. }
			| Block::StonePressurePlate { .. } => true,
			Block::RedstoneComparator { comparator }
				if BlockFace::from(comparator.facing) == side =>
			{
				true
			}
			Block::RedstoneRepeater { repeater } if BlockFace::from(repeater.facing) == side => {
				true
			}
			_ => false,
		}
	}

	fn provides_strong_power(&self, block: Block, side: BlockFace) -> bool {
		match block {
			Block::RedstoneWallTorch { .. } | Block::RedstoneTorch { .. }
				if matches!(side, BlockFace::Bottom) =>
			{
				true
			}
			Block::StonePressurePlate { .. } if matches!(side, BlockFace::Top) => true,
			Block::Lever { lever } => match side {
				BlockFace::Top => matches!(lever.face, LeverFace::Floor),
				BlockFace::Bottom => matches!(lever.face, LeverFace::Ceiling),
				_ => {
					matches!(lever.face, LeverFace::Wall) && side.direction() == Some(lever.facing)
				}
			},
			Block::StoneButton { button } => match side {
				BlockFace::Top => matches!(button.face, ButtonFace::Floor),
				BlockFace::Bottom => matches!(button.face, ButtonFace::Ceiling),
				_ => {
					matches!(button.face, ButtonFace::Wall)
						&& side.direction() == Some(button.facing)
				}
			},
			Block::RedstoneRepeater { .. } | Block::RedstoneComparator { .. } => {
				self.provides_weak_power(block, side)
			}
			_ => false,
		}
	}

	fn get_redstone_links(
		&mut self,
		block: Block,
		side: BlockFace,
		pos: BlockPos,
		link_ty: LinkType,
		distance: u8,
		start_node: NodeIndex,
		search_wire: bool,
	) {
		if block.is_solid() {
			for side in BlockFace::values() {
				let pos = pos.offset(side);
				let block = self.world.get_block(pos);
				if self.provides_strong_power(block, side) {
					self.graph.add_edge(
						self.pos_map[&pos],
						start_node,
						CompileLink::new(link_ty, distance),
					);
				}

				if let Block::RedstoneWire { wire } = block {
					if !search_wire {
						continue;
					}

					match side {
						BlockFace::Top => self.search_wire(start_node, pos, link_ty, distance),
						BlockFace::Bottom => {}
						_ => {
							let direction = side.direction().unwrap();
							if search_wire
								&& !wire::get_current_side(
									wire::get_regulated_sides(wire, self.world, pos),
									!direction,
								)
								.is_none()
							{
								self.search_wire(start_node, pos, link_ty, distance);
							}
						}
					}
				}
			}
		} else if self.provides_weak_power(block, side) {
			self.graph.add_edge(
				self.pos_map[&pos],
				start_node,
				CompileLink::new(link_ty, distance),
			);
		} else if let Block::RedstoneWire { wire } = block {
			match side {
				BlockFace::Top => self.search_wire(start_node, pos, link_ty, distance),
				BlockFace::Bottom => {}
				_ => {
					let direction = side.direction().unwrap();
					if search_wire
						&& !wire::get_current_side(
							wire::get_regulated_sides(wire, self.world, pos),
							!direction,
						)
						.is_none()
					{
						self.search_wire(start_node, pos, link_ty, distance);
					}
				}
			}
		}
	}

	fn search_wire(
		&mut self,
		start_node: NodeIndex,
		root_pos: BlockPos,
		link_ty: LinkType,
		mut distance: u8,
	) {
		let mut queue = VecDeque::<BlockPos>::new();
		let mut discovered = FxHashMap::default();

		discovered.insert(root_pos, distance);
		queue.push_back(root_pos);

		while !queue.is_empty() {
			let pos = queue.pop_front().unwrap();
			distance = discovered[&pos];

			if distance > 15 {
				continue;
			}

			let up_pos = pos.offset(BlockFace::Top);
			let up_block = self.world.get_block(up_pos);

			for side in BlockFace::values() {
				let neighbor_pos = pos.offset(side);
				let neighbor = self.world.get_block(neighbor_pos);

				self.get_redstone_links(
					neighbor,
					side,
					neighbor_pos,
					link_ty,
					distance,
					start_node,
					false,
				);
				if is_wire(self.world, neighbor_pos) && !discovered.contains_key(&neighbor_pos) {
					queue.push_back(neighbor_pos);
					discovered.insert(neighbor_pos, discovered[&pos] + 1);
				}

				if side.is_horizontal() {
					if !up_block.is_solid() && !neighbor.is_transparent() {
						let neighbor_up_pos = neighbor_pos.offset(BlockFace::Top);
						if is_wire(self.world, neighbor_up_pos)
							&& !discovered.contains_key(&neighbor_up_pos)
						{
							queue.push_back(neighbor_up_pos);
							discovered.insert(neighbor_up_pos, discovered[&pos] + 1);
						}
					}

					if !neighbor.is_solid() {
						let neighbor_down_pos = neighbor_pos.offset(BlockFace::Bottom);
						if is_wire(self.world, neighbor_down_pos)
							&& !discovered.contains_key(&neighbor_down_pos)
						{
							queue.push_back(neighbor_down_pos);
							discovered.insert(neighbor_down_pos, discovered[&pos] + 1);
						}
					}
				}
			}
		}
	}

	fn search_diode_inputs(&mut self, id: NodeIndex, pos: BlockPos, facing: BlockDirection) {
		let input_pos = pos.offset(facing.into());
		let input_block = self.world.get_block(input_pos);
		self.get_redstone_links(
			input_block,
			facing.into(),
			input_pos,
			LinkType::Default,
			0,
			id,
			true,
		);
	}

	fn search_repeater_side(&mut self, id: NodeIndex, pos: BlockPos, side: BlockDirection) {
		let side_pos = pos.offset(side.into());
		let side_block = self.world.get_block(side_pos);
		if vmm_redstone::is_diode(side_block) && self.provides_weak_power(side_block, side.into()) {
			self.graph
				.add_edge(self.pos_map[&side_pos], id, CompileLink::side(0));
		}
	}

	fn search_comparator_side(&mut self, id: NodeIndex, pos: BlockPos, side: BlockDirection) {
		let side_pos = pos.offset(side.into());
		let side_block = self.world.get_block(side_pos);
		if (vmm_redstone::is_diode(side_block) && self.provides_weak_power(side_block, side.into()))
			|| matches!(side_block, Block::RedstoneBlock {})
		{
			self.graph
				.add_edge(self.pos_map[&side_pos], id, CompileLink::side(0));
		} else if matches!(side_block, Block::RedstoneWire { .. }) {
			self.search_wire(id, side_pos, LinkType::Side, 0);
		}
	}

	fn search_node(&mut self, id: NodeIndex, (pos, block_id): (BlockPos, u32)) {
		match Block::from_id(block_id) {
			Block::RedstoneTorch { .. } => {
				let bottom_pos = pos.offset(BlockFace::Bottom);
				let bottom_block = self.world.get_block(bottom_pos);
				self.get_redstone_links(
					bottom_block,
					BlockFace::Top,
					bottom_pos,
					LinkType::Default,
					0,
					id,
					true,
				);
			}
			Block::RedstoneWallTorch { facing, .. } => {
				let wall_pos = pos.offset((!facing).into());
				let wall_block = self.world.get_block(wall_pos);
				self.get_redstone_links(
					wall_block,
					(!facing).into(),
					wall_pos,
					LinkType::Default,
					0,
					id,
					true,
				);
			}
			Block::RedstoneComparator { comparator } => {
				let facing = comparator.facing;

				self.search_comparator_side(id, pos, facing.rotate_cw());
				self.search_comparator_side(id, pos, facing.rotate_ccw());

				let input_pos = pos.offset(facing.into());
				let input_block = self.world.get_block(input_pos);
				if comparator::has_override(input_block) {
					self.graph
						.add_edge(self.pos_map[&input_pos], id, CompileLink::default(0));
				} else {
					self.search_diode_inputs(id, pos, facing);
				}
			}
			Block::RedstoneRepeater {
				repeater: RedstoneRepeater { facing, .. },
			} => {
				self.search_diode_inputs(id, pos, facing);
				self.search_repeater_side(id, pos, facing.rotate_cw());
				self.search_repeater_side(id, pos, facing.rotate_ccw());
			}
			Block::RedstoneWire { .. } => self.search_wire(id, pos, LinkType::Default, 0),
			Block::RedstoneLamp { .. } | Block::IronTrapdoor { .. } | Block::NoteBlock { .. } => {
				for face in BlockFace::values() {
					let neighbor_pos = pos.offset(face);
					let neighbor_block = self.world.get_block(neighbor_pos);
					self.get_redstone_links(
						neighbor_block,
						face,
						neighbor_pos,
						LinkType::Default,
						0,
						id,
						true,
					);
				}
			}
			_ => {}
		}
	}

	fn search(&mut self) {
		for idx in (0..self.graph.node_bound()).map(NodeIndex::new) {
			if !self.graph.contains_node(idx) {
				continue;
			}

			let node = &self.graph[idx];
			self.search_node(idx, node.block.unwrap());
		}
	}
}

fn is_wire(world: &impl World, pos: BlockPos) -> bool {
	matches!(world.get_block(pos), Block::RedstoneWire { .. })
}
