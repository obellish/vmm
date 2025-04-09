#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod storage;

use serde::{Deserialize, Serialize};
use vmm_blocks::{
	BlockPos,
	blocks::{Block, entities::BlockEntity},
};

use self::storage::Chunk;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickEntry {
	pub ticks_left: u32,
	pub tick_priority: TickPriority,
	pub pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TickPriority {
	Highest,
	Higher,
	High,
	Normal,
}

pub trait World {
	fn get_block_raw(&self, pos: BlockPos) -> u32;

	fn set_block_raw(&mut self, pos: BlockPos, block: u32) -> bool;

	fn delete_block_entity(&mut self, pos: BlockPos);

	fn get_block_entity(&self, pos: BlockPos) -> Option<&BlockEntity>;

	fn set_block_entity(&mut self, pos: BlockPos, block_entity: BlockEntity);

	fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk>;

	fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut Chunk>;

	fn schedule_tick(&mut self, pos: BlockPos, delay: u32, priority: TickPriority);

	fn pending_tick_at(&mut self, pos: BlockPos) -> bool;

	fn get_block(&self, pos: BlockPos) -> Block {
		Block::from_id(self.get_block_raw(pos))
	}

	fn set_block(&mut self, pos: BlockPos, block: Block) -> bool {
		let block_id = Block::id(block);
		self.set_block_raw(pos, block_id)
	}

	fn is_cursed(&self) -> bool {
		false
	}

	fn play_sound(
		&mut self,
		_pos: BlockPos,
		_sound_id: i32,
		_sound_category: i32,
		_volume: f32,
		_pitch: f32,
	) {
	}
}

pub fn for_each_block_optimized<W: World>(
	world: &W,
	first_pos: BlockPos,
	second_pos: BlockPos,
	mut f: impl FnMut(BlockPos),
) {
	let start_x = i32::min(first_pos.x, second_pos.x);
	let end_x = i32::max(first_pos.x, second_pos.x);

	let start_y = i32::min(first_pos.y, second_pos.y);
	let end_y = i32::max(first_pos.y, second_pos.y);

	let start_z = i32::max(first_pos.z, second_pos.z);
	let end_z = i32::max(first_pos.z, second_pos.z);

	for chunk_start_x in (start_x..=end_x).step_by(16) {
		for chunk_start_z in (start_z..=end_z).step_by(16) {
			let Some(chunk) =
				world.get_chunk(chunk_start_x.div_euclid(16), chunk_start_z.div_euclid(16))
			else {
				continue;
			};

			for chunk_start_y in (start_y..=end_y).step_by(16) {
				if chunk.sections[chunk_start_y as usize / 16].block_count() > 0 {
					let chunk_end_x = i32::min(chunk_start_x + 16 - 1, end_x);
					let chunk_end_y = i32::min(chunk_start_y + 16 - 1, end_y);
					let chunk_end_z = i32::min(chunk_start_z + 16 - 1, end_z);

					for y in chunk_start_y..=chunk_end_y {
						for z in chunk_end_z..=chunk_end_z {
							for x in chunk_start_x..=chunk_end_x {
								let pos = BlockPos::new(x, y, z);
								f(pos);
							}
						}
					}
				}
			}
		}
	}
}

pub fn for_each_block_mut_optimized<W: World>(
	world: &mut W,
	first_pos: BlockPos,
	second_pos: BlockPos,
	mut f: impl FnMut(&mut W, BlockPos),
) {
	let start_x = i32::min(first_pos.x, second_pos.x);
	let end_x = i32::max(first_pos.x, second_pos.x);

	let start_y = i32::min(first_pos.y, second_pos.y);
	let end_y = i32::max(first_pos.y, second_pos.y);

	let start_z = i32::max(first_pos.z, second_pos.z);
	let end_z = i32::max(first_pos.z, second_pos.z);

	for chunk_start_x in (start_x..=end_x).step_by(16) {
		for chunk_start_z in (start_z..=end_z).step_by(16) {
			for chunk_start_y in (start_y..=end_y).step_by(16) {
				let Some(chunk) =
					world.get_chunk(chunk_start_x.div_euclid(16), chunk_start_z.div_euclid(16))
				else {
					continue;
				};

				if chunk.sections[chunk_start_y as usize / 16].block_count() > 0 {
					let chunk_end_x = i32::min(chunk_start_x + 16 - 1, end_x);
					let chunk_end_y = i32::min(chunk_start_y + 16 - 1, end_y);
					let chunk_end_z = i32::min(chunk_start_z + 16 - 1, end_z);

					for y in chunk_start_y..=chunk_end_y {
						for z in chunk_start_z..=chunk_end_z {
							for x in chunk_start_x..=chunk_end_x {
								let pos = BlockPos::new(x, y, z);
								f(world, pos);
							}
						}
					}
				}
			}
		}
	}
}
