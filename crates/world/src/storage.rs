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
