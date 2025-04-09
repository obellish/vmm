use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	mem,
};

use rustc_hash::FxHashMap;
use vmm_blocks::{BlockPos, blocks::entities::BlockEntity};
#[cfg(feature = "networking")]
use vmm_network::packets::{
	PacketEncoder, PalettedContainer,
	clientbound::{
		CChunkData, CChunkDataBlockEntity, CChunkDataSection, CUpdateSectionBlocks,
		CUpdateSectionBlocksRecord, ClientBoundPacket,
	},
};

#[derive(Clone)]
pub struct BitBuffer {
	bits_per_entry: u64,
	entries_per_long: u64,
	entries: usize,
	mask: u64,
	longs: Vec<u64>,
	fast_arr_idx: fn(word_idx: usize) -> usize,
}

impl BitBuffer {
	fn find_fast_arr_idx_fn(entries_per_long: usize) -> fn(word_idx: usize) -> usize {
		fn fast_arr_idx<const N: usize>(word_idx: usize) -> usize {
			word_idx / N
		}

		match entries_per_long {
			16 => fast_arr_idx::<16>,
			12 => fast_arr_idx::<12>,
			10 => fast_arr_idx::<10>,
			9 => fast_arr_idx::<9>,
			8 => fast_arr_idx::<8>,
			7 => fast_arr_idx::<7>,
			6 => fast_arr_idx::<6>,
			5 => fast_arr_idx::<5>,
			4 => fast_arr_idx::<4>,
			_ => unreachable!("entries_per_long cannot be {entries_per_long}"),
		}
	}

	#[must_use]
	pub fn new(bits_per_entry: u8, entries: usize) -> Self {
		let entries_per_long = 64 / u64::from(bits_per_entry);
		let longs_len = entries.div_ceil(entries_per_long as usize);
		let longs = vec![0; longs_len];

		Self {
			bits_per_entry: u64::from(bits_per_entry),
			longs,
			entries,
			entries_per_long,
			mask: (1 << bits_per_entry) - 1,
			fast_arr_idx: Self::find_fast_arr_idx_fn(entries_per_long as usize),
		}
	}

	fn load(entries: usize, bits_per_entry: u8, longs: Vec<u64>) -> Self {
		let entries_per_long = 64 / u64::from(bits_per_entry);
		Self {
			bits_per_entry: u64::from(bits_per_entry),
			longs,
			entries,
			entries_per_long,
			mask: (1 << bits_per_entry) - 1,
			fast_arr_idx: Self::find_fast_arr_idx_fn(entries_per_long as usize),
		}
	}

	#[must_use]
	pub fn get(&self, word_idx: usize) -> u32 {
		let arr_idx = (self.fast_arr_idx)(word_idx);
		let sub_idx =
			(word_idx as u64 - arr_idx as u64 * self.entries_per_long) * self.bits_per_entry;
		let word = (self.longs[arr_idx] >> sub_idx) & self.mask;
		word as u32
	}

	pub fn set(&mut self, word_idx: usize, word: u32) {
		let arr_idx = (self.fast_arr_idx)(word_idx);
		let sub_idx =
			(word_idx as u64 - arr_idx as u64 * self.entries_per_long) * self.bits_per_entry;
		let mask = !(self.mask << sub_idx);
		self.longs[arr_idx] = (self.longs[arr_idx] & mask) | (u64::from(word) << sub_idx);
	}
}

impl Debug for BitBuffer {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("BitBuffer")
			.field("bits_per_entry", &self.bits_per_entry)
			.field("entries", &self.entries)
			.field("entries_per_long", &self.entries_per_long)
			.field("mask", &self.mask)
			.finish_non_exhaustive()
	}
}

#[derive(Debug, Clone)]
pub struct PalettedBitBuffer {
	data: BitBuffer,
	palette: Vec<u32>,
	max_entries: u32,
	use_palette: bool,
	direct_threshold: u64,
}

impl PalettedBitBuffer {
	#[must_use]
	pub fn new(entries: usize, direct_threshold: u64) -> Self {
		let palette = vec![0];
		Self {
			data: BitBuffer::new(4, entries),
			palette,
			max_entries: 16,
			use_palette: true,
			direct_threshold,
		}
	}

	fn load(
		entries: usize,
		bits_per_entry: u8,
		longs: Vec<u64>,
		palette: Vec<u32>,
		direct_threshold: u64,
	) -> Self {
		Self {
			data: BitBuffer::load(entries, bits_per_entry, longs),
			palette,
			use_palette: bits_per_entry < 9,
			max_entries: 1 << bits_per_entry,
			direct_threshold,
		}
	}

	fn resize_buffer(&mut self) {
		assert!(
			self.use_palette,
			"the buffer should never be resizing if it's already using the global palette"
		);

		let old_bits_per_entry = self.data.bits_per_entry;

		let new_bits = if old_bits_per_entry + 1 >= self.direct_threshold {
			self.max_entries = 1 << 15;
			self.use_palette = false;
			15
		} else {
			self.max_entries <<= 1;
			old_bits_per_entry as u8 + 1
		};

		let mut old_buffer = BitBuffer::new(new_bits, self.data.entries);
		mem::swap(&mut self.data, &mut old_buffer);

		if matches!(new_bits, 15) {
			for entry_idx in 0..old_buffer.entries {
				let entry = self.palette[old_buffer.get(entry_idx) as usize];
				self.data.set(entry_idx, entry);
			}

			self.palette = Vec::new();
		} else {
			for entry_idx in 0..old_buffer.entries {
				let entry = old_buffer.get(entry_idx);
				self.data.set(entry_idx, entry);
			}
		}
	}

	#[must_use]
	pub fn get(&self, index: usize) -> u32 {
		if self.use_palette {
			self.palette[self.data.get(index) as usize]
		} else {
			self.data.get(index)
		}
	}

	pub fn set(&mut self, index: usize, value: u32) {
		if self.use_palette {
			if let Some(palette_index) = self.palette.iter().position(|x| x == &value) {
				self.data.set(index, palette_index as u32);
			} else {
				if self.palette.len() + 1 > self.max_entries as usize {
					self.resize_buffer();
					self.set(index, value);
					return;
				}

				let palette_index = self.palette.len();
				self.palette.push(value);
				self.data.set(index, palette_index as u32);
			}
		} else {
			self.data.set(index, value);
		}
	}

	#[must_use]
	pub const fn entries(&self) -> usize {
		self.data.entries
	}

	#[cfg(feature = "networking")]
	fn encode_packet(&self) -> PalettedContainer {
		if matches!((self.use_palette, self.palette.len()), (true, 1)) {
			PalettedContainer {
				bits_per_entry: 0,
				data_array: vec![0],
				palette: Some(vec![self.palette[0] as i32]),
			}
		} else {
			PalettedContainer {
				bits_per_entry: self.data.bits_per_entry as u8,
				data_array: self.data.longs.clone(),
				palette: self
					.use_palette
					.then(|| self.palette.clone().into_iter().map(|x| x as i32).collect()),
			}
		}
	}
}

#[derive(Clone)]
pub struct ChunkSection {
	buffer: PalettedBitBuffer,
	block_count: u32,
	#[cfg(feature = "networking")]
	multi_block: CUpdateSectionBlocks,
	changed_blocks: [i16; 16 * 16 * 16],
	changed: bool,
}

impl ChunkSection {
	#[must_use]
	pub fn from_raw(
		data: Vec<u64>,
		bits_per_block: u8,
		palette: Vec<u32>,
		block_count: u32,
	) -> Self {
		let entries = 16 * 16 * 16;
		let buffer = PalettedBitBuffer::load(entries, bits_per_block, data, palette, 9);
		Self {
			buffer,
			block_count,
			#[cfg(feature = "networking")]
			multi_block: CUpdateSectionBlocks {
				chunk_x: 0,
				chunk_y: 0,
				chunk_z: 0,
				records: Vec::new(),
			},
			changed_blocks: [-1; 16 * 16 * 16],
			changed: false,
		}
	}

	const fn get_index(x: u32, y: u32, z: u32) -> usize {
		((y << 8) | (z << 4) | x) as usize
	}

	#[must_use]
	pub fn get_block(&self, x: u32, y: u32, z: u32) -> u32 {
		let idx = Self::get_index(x, y, z);
		if self.changed_blocks[idx] >= 0 {
			self.changed_blocks[idx] as u32
		} else {
			self.buffer.get(idx)
		}
	}

	pub fn set_block(&mut self, x: u32, y: u32, z: u32, block: u32) -> bool {
		let old_block = self.get_block(x, y, z);
		if matches!(old_block, 0) && !matches!(block, 0) {
			self.block_count += 1;
		} else if matches!(block, 0) && !matches!(old_block, 0) {
			self.block_count -= 1;
		}

		let idx = Self::get_index(x, y, z);
		let changed = old_block != block;
		if changed {
			self.changed = true;
			self.changed_blocks[idx] = block as i16;
		}

		changed
	}

	#[allow(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn data(&self) -> &[u64] {
		&self.buffer.data.longs
	}

	#[allow(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn palette(&self) -> &[u32] {
		&self.buffer.palette
	}

	#[must_use]
	pub const fn bits_per_block(&self) -> u8 {
		self.buffer.data.bits_per_entry as u8
	}

	#[must_use]
	pub const fn block_count(&self) -> u32 {
		self.block_count
	}

	fn compress(&mut self) {
		let mut new_buffer = PalettedBitBuffer::new(4096, 9);
		for i in 0..4096 {
			new_buffer.set(i, self.buffer.get(i));
		}

		self.buffer = new_buffer;
	}

	#[cfg(feature = "networking")]
	fn encode_packet(&self) -> CChunkDataSection {
		CChunkDataSection {
			block_count: self.block_count() as i16,
			block_states: self.buffer.encode_packet(),
			biomes: PalettedContainer {
				bits_per_entry: 0,
				data_array: Vec::new(),
				palette: Some(vec![0]),
			},
		}
	}

	fn flush(&mut self) {
		if self.changed {
			for (i, block) in self.changed_blocks.iter().copied().enumerate() {
				if block >= 0 {
					self.buffer.set(i, block as u32);
				}
			}
		}
	}

	#[cfg(feature = "networking")]
	fn multi_block(&mut self, chunk_x: i32, chunk_y: u32, chunk_z: i32) -> &CUpdateSectionBlocks {
		self.multi_block.chunk_x = chunk_x;
		self.multi_block.chunk_y = chunk_y;
		self.multi_block.chunk_z = chunk_z;

		if self.changed {
			for (i, block) in self.changed_blocks.iter().copied().enumerate() {
				if block >= 0 {
					self.buffer.set(i, block as u32);
					self.multi_block.records.push(CUpdateSectionBlocksRecord {
						block_id: block as u32,
						x: (i & 0xF) as u8,
						y: (i >> 8) as u8,
						z: ((i & 0xF0) >> 4) as u8,
					});
				}
			}

			self.changed = false;
			self.changed_blocks = [-1; 16 * 16 * 16];
		}

		&self.multi_block
	}
}

impl Default for ChunkSection {
	fn default() -> Self {
		Self {
			buffer: PalettedBitBuffer::new(4096, 9),
			block_count: 0,
			#[cfg(feature = "networking")]
			multi_block: CUpdateSectionBlocks {
				chunk_x: 0,
				chunk_y: 0,
				chunk_z: 0,
				records: Vec::new(),
			},
			changed_blocks: [-1; 16 * 16 * 16],
			changed: false,
		}
	}
}

#[derive(Clone)]
pub struct Chunk {
	pub sections: Vec<ChunkSection>,
	pub x: i32,
	pub z: i32,
	pub block_entities: FxHashMap<BlockPos, BlockEntity>,
}

impl Chunk {
	#[cfg(feature = "networking")]
    #[must_use]
	pub fn encode_packet(&self) -> PacketEncoder {
		let block_height = self.sections.len() * 16;

		let heightmap_bits = (32 - ((block_height as u32 + 1) - 1).leading_zeros()) as u8;
		let mut heightmap_buffer = BitBuffer::new(heightmap_bits, 16 * 16);
		for x in 0..16 {
			for z in 0..16 {
				heightmap_buffer.set((x * 16) + z, self.get_top_most_block(x as u32, z as u32));
			}
		}

		let mut chunk_sections = Vec::new();
		for section in &self.sections {
			chunk_sections.push(section.encode_packet());
		}

		let mut heightmaps = nbt::Map::new();
		let heightmap_longs = heightmap_buffer
			.longs
			.into_iter()
			.map(|x| x as i64)
			.collect::<Vec<_>>();
		heightmaps.insert(
			"MOTION_BLOCKING".to_owned(),
			nbt::Value::LongArray(heightmap_longs),
		);

		let mut block_entities = Vec::new();
		for (pos, block_entity) in &self.block_entities {
			if let Some(nbt) = block_entity.to_nbt(true) {
				block_entities.push(CChunkDataBlockEntity {
					x: pos.x as i8,
					z: pos.z as i8,
					y: pos.y as i16,
					data: nbt.content,
					ty: block_entity.ty(),
				});
			}
		}

		CChunkData {
			chunk_sections,
			chunk_x: self.x,
			chunk_z: self.z,
			heightmaps,
			block_entities,
		}
		.encode()
	}

	#[cfg(feature = "networking")]
	#[must_use]
	pub fn encode_empty_packet(x: i32, z: i32, num_sections: usize) -> PacketEncoder {
		CChunkData {
			chunk_sections: (0..num_sections)
				.map(|_| CChunkDataSection {
					block_count: 0,
					block_states: PalettedContainer {
						bits_per_entry: 0,
						data_array: vec![0],
						palette: Some(vec![0]),
					},
					biomes: PalettedContainer {
						bits_per_entry: 0,
						palette: Some(vec![0]),
						data_array: vec![0],
					},
				})
				.collect(),
			chunk_x: x,
			chunk_z: z,
			heightmaps: nbt::Map::new(),
			block_entities: Vec::new(),
		}
		.encode()
	}

	#[cfg(feature = "networking")]
	fn get_top_most_block(&self, x: u32, z: u32) -> u32 {
		let mut top_most = 0;
		for (section_y, section) in self.sections.iter().enumerate() {
			for y in (0..16).rev() {
				let block_state = section.get_block(x, y, z);
				if !matches!(block_state, 0) && top_most < y + section_y as u32 * 16 {
					top_most = section_y as u32 * 16;
				}
			}
		}

		top_most
	}

	pub fn set_block(&mut self, x: u32, y: u32, z: u32, block_id: u32) -> bool {
		let section_y = (y >> 4) as usize;
		let section = &mut self.sections[section_y];
		section.set_block(x, y & 0xF, z, block_id)
	}

	#[must_use]
	pub fn get_block(&self, x: u32, y: u32, z: u32) -> u32 {
		let section_y = (y / 16) as usize;
		match self.sections.get(section_y) {
			None => 0,
			Some(section) => section.get_block(x, y & 0xF, z),
		}
	}

	#[must_use]
	pub fn get_block_entity(&self, pos: BlockPos) -> Option<&BlockEntity> {
		self.block_entities.get(&pos)
	}

	pub fn delete_block_entity(&mut self, pos: BlockPos) {
		self.block_entities.remove(&pos);
	}

	pub fn set_block_entity(&mut self, pos: BlockPos, block_entity: BlockEntity) {
		self.block_entities.insert(pos, block_entity);
	}

	pub fn compress(&mut self) {
		self.sections.iter_mut().for_each(ChunkSection::compress);
	}

	#[must_use]
	pub fn empty(x: i32, z: i32, num_sections: usize) -> Self {
		Self {
			sections: (0..num_sections).map(|_| ChunkSection::default()).collect(),
			x,
			z,
			block_entities: FxHashMap::default(),
		}
	}

	#[cfg(feature = "networking")]
	pub fn multi_blocks(&mut self) -> impl Iterator<Item = &CUpdateSectionBlocks> {
		let x = self.x;
		let z = self.z;
		self.sections
			.iter_mut()
			.enumerate()
			.filter_map(move |(y, section)| {
				section
					.changed
					.then(move || section.multi_block(x, y as u32, z))
			})
	}

	#[cfg(feature = "networking")]
	pub fn reset_multi_blocks(&mut self) {
		for section in &mut self.sections {
			section.multi_block.records.clear();
		}
	}

	pub fn flush(&mut self) {
		for section in &mut self.sections {
			section.flush();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::BitBuffer;

	#[test]
	fn bitbuffer_format() {
		let entries = [
			1, 2, 2, 3, 4, 4, 5, 6, 6, 4, 8, 0, 7, 4, 3, 13, 15, 16, 9, 14, 10, 12, 0, 2,
		];

		let mut buffer = BitBuffer::new(5, 24);
		for (i, entry) in entries.iter().copied().enumerate() {
			buffer.set(i, entry);
		}

		assert_eq!(buffer.longs[0], 0x0020_8631_4841_8841);
		assert_eq!(buffer.longs[1], 0x0101_8A72_60F6_8C87);
	}
}
