use vmm_blocks::{
	BlockFace, BlockPos,
	blocks::{Block, Instrument},
};
use vmm_world::World;

#[expect(clippy::approx_constant)]
const PITCHES_TABLE: [f32; 25] = [
	0.5,
	0.529_731_5,
	0.561_231,
	0.594_603_54,
	0.629_960_54,
	0.667_419_9,
	0.707_106_77,
	0.749_153_55,
	0.793_700_5,
	0.840_896_4,
	0.890_898_7,
	0.943_874_3,
	1.0,
	1.059_463_1,
	1.122_462,
	1.189_207_1,
	1.259_921_1,
	1.334_839_8,
	1.414_213_5,
	1.498_307_1,
	1.587_401,
	1.681_792_9,
	1.781_797_4,
	1.887_748_6,
	2.0,
];

pub fn is_noteblock_unblocked(world: &impl World, pos: BlockPos) -> bool {
	matches!(world.get_block(pos.offset(BlockFace::Top)), Block::Air {})
}

pub fn get_noteblock_instrument(world: &impl World, pos: BlockPos) -> Instrument {
	Instrument::from_block_below(world.get_block(pos.offset(BlockFace::Bottom)))
}

pub fn play_note(world: &mut impl World, pos: BlockPos, instrument: Instrument, note: u32) {
	world.play_sound(
		pos,
		instrument.to_sound_id(),
		2,
		3.0,
		PITCHES_TABLE[note as usize],
	);
}
