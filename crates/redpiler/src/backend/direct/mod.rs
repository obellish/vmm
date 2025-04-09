mod compile;
mod node;
mod tick;
mod update;

use std::{
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

impl JitBackend for DirectBackend {
	fn inspect(&mut self, pos: BlockPos) {
		let Some(node_id) = self.pos_map.get(&pos) else {
			debug!("could not find node at pos {pos}");
			return;
		};

		debug!("Node {node_id:?}: {:#?}", self.nodes[*node_id]);
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
