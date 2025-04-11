mod compile;
mod node;
mod tick;
mod update;

use std::{
	borrow::Cow,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	mem,
	sync::Arc,
};

use rustc_hash::FxHashMap;
use tracing::{debug, warn};
use vmm_blocks::{
	BlockPos,
	blocks::{Block, ComparatorMode, Instrument, entities::BlockEntity},
};
use vmm_redstone::{bool_to_signal_strength, noteblock};
use vmm_world::{TickEntry, TickPriority, World};

use self::node::{Node, NodeId, NodeType, Nodes};
use super::JitBackend;
use crate::{CompilerOptions, TaskMonitor, block_powered_mut, compile_graph::CompileGraph};

#[derive(Default)]
pub struct DirectBackend {
	nodes: Nodes,
	blocks: Vec<Option<(BlockPos, Block)>>,
	pos_map: FxHashMap<BlockPos, NodeId>,
	scheduler: TickScheduler,
	events: Vec<Event>,
	noteblock_info: Vec<(BlockPos, Instrument, u32)>,
}

impl DirectBackend {
	fn schedule_tick(&mut self, node_id: NodeId, delay: usize, priority: TickPriority) {
		self.scheduler.schedule_tick(node_id, delay, priority);
	}

	fn set_node(&mut self, node_id: NodeId, powered: bool, new_power: u8) {
		let node = &mut self.nodes[node_id];
		let old_power = node.output_power;

		node.changed = true;
		node.powered = powered;
		node.output_power = new_power;

		for i in 0..node.updates.len() {
			let node = &self.nodes[node_id];
			let update_link = unsafe { *node.updates.get_unchecked(i) };
			let side = update_link.side();
			let distance = update_link.signal_strength();
			let update = update_link.node();

			let update_ref = &mut self.nodes[update];
			let inputs = if side {
				&mut update_ref.side_inputs
			} else {
				&mut update_ref.default_inputs
			};

			let old_power = old_power.saturating_sub(distance);
			let new_power = new_power.saturating_sub(distance);

			if old_power == new_power {
				continue;
			}

			unsafe {
				*inputs
					.signal_strength_counts
					.get_unchecked_mut(old_power as usize) -= 1;
				*inputs
					.signal_strength_counts
					.get_unchecked_mut(new_power as usize) += 1;
			}

			self::update::update_node(
				&mut self.scheduler,
				&mut self.events,
				&mut self.nodes,
				node_id,
			);
		}
	}
}

impl Display for DirectBackend {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("digraph {\n")?;

		for (id, node) in self.nodes.inner().iter().enumerate() {
			if matches!(node.ty, NodeType::Wire) {
				continue;
			}

			let label = match node.ty {
				NodeType::Repeater { delay, .. } => Cow::Owned(format!("Repeater({delay})")),
				NodeType::Torch => Cow::Borrowed("Torch"),
				NodeType::Comparator { mode, .. } => Cow::Owned(format!(
					"Comparator({})",
					match mode {
						ComparatorMode::Compare => "Cmp",
						ComparatorMode::Subtract => "Sub",
					}
				)),
				NodeType::Lamp => Cow::Borrowed("Lamp"),
				NodeType::Button => Cow::Borrowed("Button"),
				NodeType::Lever => Cow::Borrowed("Lever"),
				NodeType::PressurePlate => Cow::Borrowed("PressurePlate"),
				NodeType::Trapdoor => Cow::Borrowed("Trapdoor"),
				NodeType::Wire => Cow::Borrowed("Wire"),
				NodeType::Constant => Cow::Owned(format!("Constant({})", node.output_power)),
				NodeType::NoteBlock { .. } => Cow::Borrowed("NoteBlock"),
			};

			let pos = if let Some((pos, _)) = self.blocks[id] {
				Cow::Owned(format!("{}, {}, {}", pos.x, pos.y, pos.z))
			} else {
				Cow::Borrowed("No Pos")
			};

			f.write_str("    n")?;
			Display::fmt(&id, f)?;
			f.write_str(" [ label = \"")?;
			f.write_str(&label)?;
			f.write_str("\\n(")?;
			f.write_str(&pos)?;
			f.write_str("\" ];\n")?;

			for link in &node.updates {
				let out_index = link.node().index();
				let distance = link.signal_strength();
				let color = if link.side() { ",color=\"blue\"" } else { "" };

				f.write_str("    n")?;
				Display::fmt(&id, f)?;
				f.write_str(" -> n")?;
				Display::fmt(&out_index, f)?;
				f.write_str(" [ label = \"")?;
				Display::fmt(&distance, f)?;
				f.write_str("\"")?;
				f.write_str(color)?;
				f.write_str(" ];\n")?;
			}
		}

		f.write_char('}')
	}
}

impl JitBackend for DirectBackend {
	fn inspect(&mut self, pos: BlockPos) {
		let Some(node_id) = self.pos_map.get(&pos) else {
			debug!("could not find node at pos {pos}");
			return;
		};

		debug!("Node {node_id:?}: {:#?}", self.nodes[*node_id]);
	}

	fn reset<W: World>(&mut self, world: &mut W, io_only: bool) {
		self.scheduler.reset(world, &self.blocks);

		let nodes = std::mem::take(&mut self.nodes);

		for (i, node) in nodes.into_inner().iter().enumerate() {
			let Some((pos, block)) = self.blocks[i] else {
				continue;
			};

			if matches!(node.ty, NodeType::Comparator { .. }) {
				let block_entity = BlockEntity::Comparator {
					output_strength: node.output_power,
				};
				world.set_block_entity(pos, block_entity);
			}

			if io_only && !node.is_io {
				world.set_block(pos, block);
			}
		}

		self.pos_map.clear();
		self.noteblock_info.clear();
		self.events.clear();
	}

	fn on_use_block(&mut self, pos: BlockPos) {
		let node_id = self.pos_map[&pos];
		let node = &self.nodes[node_id];
		match node.ty {
			NodeType::Button => {
				if node.powered {
					return;
				}

				self.schedule_tick(node_id, 10, TickPriority::Normal);
				self.set_node(node_id, true, 15);
			}
			NodeType::Lever => self.set_node(
				node_id,
				!node.powered,
				bool_to_signal_strength(!node.powered),
			),
			_ => warn!("tried to use a {:?} redpiler node", node.ty),
		}
	}

	fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool) {
		let node_id = self.pos_map[&pos];
		let node = &self.nodes[node_id];
		if matches!(node.ty, NodeType::PressurePlate) {
			self.set_node(node_id, powered, bool_to_signal_strength(powered));
		} else {
			warn!("tried to set pressure plate state for a {:?}", node.ty);
		}
	}

	fn tick(&mut self) {
		let mut queues = self.scheduler.queues_this_tick();

		for node_id in queues.drain_iter() {
			self.tick_node(node_id);
		}

		self.scheduler.end_tick(queues);
	}

	fn flush<W: World>(&mut self, world: &mut W, io_only: bool) {
		for event in self.events.drain(..) {
			match event {
				Event::NoteBlockPlay { noteblock_id } => {
					let (pos, instrument, note) = self.noteblock_info[noteblock_id as usize];
					noteblock::play_note(world, pos, instrument, note);
				}
			}
		}

		for (i, node) in self.nodes.inner_mut().iter_mut().enumerate() {
			let Some((pos, block)) = &mut self.blocks[i] else {
				continue;
			};

			if node.changed && (!io_only || node.is_io) {
				if let Some(powered) = block_powered_mut(block) {
					*powered = node.powered;
				}

				if let Block::RedstoneWire { wire, .. } = block {
					wire.power = node.output_power;
				}

				if let Block::RedstoneRepeater { repeater } = block {
					repeater.locked = node.locked;
				}

				world.set_block(*pos, *block);
			}

			node.changed = false;
		}
	}

	fn compile(
		&mut self,
		graph: CompileGraph,
		ticks: Vec<TickEntry>,
		options: CompilerOptions,
		monitor: Arc<TaskMonitor>,
	) {
		self::compile::compile(self, graph, ticks, options, monitor);
	}

	fn has_pending_ticks(&self) -> bool {
		self.scheduler.has_pending_ticks()
	}
}

#[derive(Default, Clone)]
#[repr(transparent)]
struct Queues([Vec<NodeId>; TickScheduler::NUM_PRIORITIES]);

impl Queues {
	fn drain_iter(&mut self) -> impl Iterator<Item = NodeId> + '_ {
		let [q0, q1, q2, q3] = &mut self.0;
		let [q0, q1, q2, q3] = [q0, q1, q2, q3].map(|q| q.drain(..));
		q0.chain(q1).chain(q2).chain(q3)
	}
}

#[derive(Default)]
struct TickScheduler {
	queues_deque: [Queues; Self::NUM_QUEUES],
	pos: usize,
}

impl TickScheduler {
	const NUM_PRIORITIES: usize = 4;
	const NUM_QUEUES: usize = 16;

	fn reset(&mut self, world: &mut impl World, blocks: &[Option<(BlockPos, Block)>]) {
		for (idx, queues) in self.queues_deque.iter().enumerate() {
			let delay = if self.pos >= idx {
				idx + Self::NUM_QUEUES
			} else {
				idx
			} - self.pos;

			for (entries, priority) in queues.0.iter().zip(Self::priorities()) {
				for node in entries {
					let Some((pos, _)) = blocks[node.index()] else {
						warn!(
							"cannot schedule tick for node {node:?} because block information is missing"
						);
						continue;
					};

					world.schedule_tick(pos, delay as u32, priority);
				}
			}
		}

		for queues in &mut self.queues_deque {
			for queue in &mut queues.0 {
				queue.clear();
			}
		}
	}

	fn schedule_tick(&mut self, node: NodeId, delay: usize, priority: TickPriority) {
		self.queues_deque[(self.pos + delay) % Self::NUM_QUEUES].0[priority as usize].push(node);
	}

	fn queues_this_tick(&mut self) -> Queues {
		self.pos = (self.pos + 1) % Self::NUM_QUEUES;
		mem::take(&mut self.queues_deque[self.pos])
	}

	fn end_tick(&mut self, mut queues: Queues) {
		for queue in &mut queues.0 {
			queue.clear();
		}

		self.queues_deque[self.pos] = queues;
	}

	const fn priorities() -> [TickPriority; Self::NUM_PRIORITIES] {
		[
			TickPriority::Highest,
			TickPriority::Higher,
			TickPriority::High,
			TickPriority::Normal,
		]
	}

	fn has_pending_ticks(&self) -> bool {
		for queue in &self.queues_deque {
			for queue in &queue.0 {
				if !queue.is_empty() {
					return true;
				}
			}
		}

		false
	}
}

#[repr(transparent)]
enum Event {
	NoteBlockPlay { noteblock_id: u16 },
}

const BOOL_INPUT_MASK: u128 = u128::from_ne_bytes([
	0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

const fn set_node(node: &mut Node, powered: bool) {
	node.powered = powered;
	node.changed = true;
}

const fn set_node_locked(node: &mut Node, locked: bool) {
	node.locked = locked;
	node.changed = true;
}

fn schedule_tick(
	scheduler: &mut TickScheduler,
	node_id: NodeId,
	node: &mut Node,
	delay: usize,
	priority: TickPriority,
) {
	node.pending_tick = true;
	scheduler.schedule_tick(node_id, delay, priority);
}

const fn get_bool_input(node: &Node) -> bool {
	!matches!(
		u128::from_le_bytes(node.default_inputs.signal_strength_counts) & BOOL_INPUT_MASK,
		0
	)
}

const fn get_bool_side(node: &Node) -> bool {
	!matches!(
		u128::from_le_bytes(node.side_inputs.signal_strength_counts) & BOOL_INPUT_MASK,
		0
	)
}

const fn last_index_positive(array: [u8; 16]) -> u32 {
	let value = u128::from_le_bytes(array);
	if matches!(value, 0) {
		0
	} else {
		15 - (value.leading_zeros() >> 3)
	}
}

const fn get_all_input(node: &Node) -> (u8, u8) {
	let input_power = last_index_positive(node.default_inputs.signal_strength_counts) as u8;
	let side_input_power = last_index_positive(node.side_inputs.signal_strength_counts) as u8;

	(input_power, side_input_power)
}

const fn calculate_comparator_output(
	mode: ComparatorMode,
	input_strength: u8,
	power_on_sides: u8,
) -> u8 {
	let difference = input_strength.wrapping_sub(power_on_sides);
	if difference <= 15 {
		match mode {
			ComparatorMode::Compare => input_strength,
			ComparatorMode::Subtract => difference,
		}
	} else {
		0
	}
}
