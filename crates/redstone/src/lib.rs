#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod comparator;
pub mod noteblock;
pub mod repeater;
pub mod wire;

use vmm_blocks::{
	BlockDirection, BlockFace, BlockPos,
	blocks::{
		Block, ButtonFace, Lever, LeverFace, RedstoneWire, StoneButton, entities::BlockEntity,
	},
};
use vmm_world::{TickPriority, World};

#[must_use]
pub const fn bool_to_signal_strength(v: bool) -> u8 {
	if v { 15 } else { 0 }
}

pub fn get_redstone_power(
	block: Block,
	world: &impl World,
	pos: BlockPos,
	facing: BlockFace,
) -> u8 {
	if block.is_solid() {
		get_max_strong_power(world, pos, true)
	} else {
		get_weak_power(block, world, pos, facing, true)
	}
}

pub fn torch_should_be_off(world: &impl World, pos: BlockPos) -> bool {
	let bottom_pos = pos.offset(BlockFace::Bottom);
	let bottom_block = world.get_block(bottom_pos);
	get_redstone_power(bottom_block, world, bottom_pos, BlockFace::Top) > 0
}

pub fn wall_torch_should_be_off(
	world: &impl World,
	pos: BlockPos,
	direction: BlockDirection,
) -> bool {
	let wall_pos = pos.offset((!direction).into());
	let wall_block = world.get_block(wall_pos);
	get_redstone_power(wall_block, world, wall_pos, (!direction).into()) > 0
}

pub fn redstone_lamp_should_be_lit(world: &impl World, pos: BlockPos) -> bool {
	for face in BlockFace::values() {
		let neighbor_pos = pos.offset(face);
		if get_redstone_power(world.get_block(neighbor_pos), world, neighbor_pos, face) > 0 {
			return true;
		}
	}

	false
}

#[must_use]
pub const fn is_diode(block: Block) -> bool {
	matches!(
		block,
		Block::RedstoneRepeater { .. } | Block::RedstoneComparator { .. }
	)
}

pub fn update(block: Block, world: &mut impl World, pos: BlockPos) {
	match block {
		Block::RedstoneWire { wire } => {
			self::wire::on_neighbor_updated(wire, world, pos);
		}
		Block::RedstoneTorch { lit } => {
			if lit == torch_should_be_off(world, pos) && !world.pending_tick_at(pos) {
				world.schedule_tick(pos, 1, TickPriority::Normal);
			}
		}
		Block::RedstoneWallTorch { lit, facing } => {
			if lit == wall_torch_should_be_off(world, pos, facing) && !world.pending_tick_at(pos) {
				world.schedule_tick(pos, 1, TickPriority::Normal);
			}
		}
		Block::RedstoneRepeater { repeater } => {
			self::repeater::on_neighbor_updated(repeater, world, pos);
		}
		Block::RedstoneComparator { comparator } => {
			self::comparator::update(comparator, world, pos);
		}
		Block::RedstoneLamp { lit } => {
			let should_be_lit = redstone_lamp_should_be_lit(world, pos);
			if lit && !should_be_lit {
				world.schedule_tick(pos, 2, TickPriority::Normal);
			} else if !lit && should_be_lit {
				world.set_block(pos, Block::RedstoneLamp { lit: true });
			}
		}
		Block::IronTrapdoor {
			facing,
			half,
			powered,
		} => {
			let should_be_powered = redstone_lamp_should_be_lit(world, pos);
			if powered != should_be_powered {
				let new_block = Block::IronTrapdoor {
					facing,
					half,
					powered: should_be_powered,
				};

				world.set_block(pos, new_block);
			}
		}
		Block::NoteBlock { note, .. } => {
			let should_be_powered = redstone_lamp_should_be_lit(world, pos);

			let Block::NoteBlock { powered, .. } = world.get_block(pos) else {
				unreachable!("underlying block changed, this should never happen");
			};

			if powered != should_be_powered {
				let instrument = self::noteblock::get_noteblock_instrument(world, pos);
				let new_block = Block::NoteBlock {
					instrument,
					note,
					powered: should_be_powered,
				};

				if should_be_powered && self::noteblock::is_noteblock_unblocked(world, pos) {
					self::noteblock::play_note(world, pos, instrument, note);
				}

				world.set_block(pos, new_block);
			}
		}
		_ => {}
	}
}

pub fn tick(block: Block, world: &mut impl World, pos: BlockPos) {
	match block {
		Block::RedstoneRepeater { repeater } => self::repeater::tick(repeater, world, pos),
		Block::RedstoneComparator { comparator } => self::comparator::tick(comparator, world, pos),
		Block::RedstoneTorch { lit } => {
			let should_be_off = torch_should_be_off(world, pos);
			if lit && should_be_off {
				world.set_block(pos, Block::RedstoneTorch { lit: false });
				update_surrounding_blocks(world, pos);
			} else if !lit && !should_be_off {
				world.set_block(pos, Block::RedstoneTorch { lit: true });
				update_surrounding_blocks(world, pos);
			}
		}
		Block::RedstoneWallTorch { lit, facing } => {
			let should_be_off = wall_torch_should_be_off(world, pos, facing);
			if lit && should_be_off {
				world.set_block(pos, Block::RedstoneWallTorch { lit: false, facing });
				update_surrounding_blocks(world, pos);
			} else if !lit && !should_be_off {
				world.set_block(pos, Block::RedstoneWallTorch { lit: true, facing });
				update_surrounding_blocks(world, pos);
			}
		}
		Block::RedstoneLamp { lit } => {
			let should_be_lit = redstone_lamp_should_be_lit(world, pos);
			if lit && !should_be_lit {
				world.set_block(pos, Block::RedstoneLamp { lit: false });
			}
		}
		Block::StoneButton { mut button } => {
			if button.powered {
				button.powered = false;
				world.set_block(pos, Block::StoneButton { button });
				update_surrounding_blocks(world, pos);
				match button.face {
					ButtonFace::Ceiling => {
						update_surrounding_blocks(world, pos.offset(BlockFace::Top));
					}
					ButtonFace::Floor => {
						update_surrounding_blocks(world, pos.offset(BlockFace::Bottom));
					}
					ButtonFace::Wall => {
						update_surrounding_blocks(world, pos.offset((!button.facing).into()));
					}
				}
			}
		}
		_ => {}
	}
}

pub fn on_use(block: Block, world: &mut impl World, pos: BlockPos) -> bool {
	match block {
		Block::RedstoneRepeater { repeater } => {
			let mut repeater = repeater;
			repeater.delay += 1;
			if repeater.delay > 4 {
				repeater.delay -= 4;
			}
			world.set_block(pos, Block::RedstoneRepeater { repeater });
			true
		}
		Block::RedstoneComparator { comparator } => {
			let mut comparator = comparator;
			comparator.mode = !comparator.mode;
			self::comparator::tick(comparator, world, pos);
			world.set_block(pos, Block::RedstoneComparator { comparator });
			true
		}
		Block::Lever { mut lever } => {
			lever.powered = !lever.powered;
			world.set_block(pos, Block::Lever { lever });
			update_surrounding_blocks(world, pos);
			match lever.face {
				LeverFace::Ceiling => update_surrounding_blocks(world, pos.offset(BlockFace::Top)),
				LeverFace::Floor => update_surrounding_blocks(world, pos.offset(BlockFace::Bottom)),
				LeverFace::Wall => {
					update_surrounding_blocks(world, pos.offset((!lever.facing).into()));
				}
			}

			true
		}
		Block::StoneButton { mut button } => {
			if !button.powered {
				button.powered = true;
				world.set_block(pos, Block::StoneButton { button });
				world.schedule_tick(pos, 10, TickPriority::Normal);
				update_surrounding_blocks(world, pos);
				match button.face {
					ButtonFace::Ceiling => {
						update_surrounding_blocks(world, pos.offset(BlockFace::Top));
					}
					ButtonFace::Floor => {
						update_surrounding_blocks(world, pos.offset(BlockFace::Bottom));
					}
					ButtonFace::Wall => {
						update_surrounding_blocks(world, pos.offset((!button.facing).into()));
					}
				}
			}

			true
		}
		Block::RedstoneWire { wire } => {
			if self::wire::is_dot(wire) || self::wire::is_cross(wire) {
				let mut new_wire = if self::wire::is_cross(wire) {
					RedstoneWire::default()
				} else {
					self::wire::make_cross(0)
				};

				new_wire.power = wire.power;
				new_wire = self::wire::get_regulated_sides(new_wire, world, pos);
				if wire != new_wire {
					world.set_block(pos, Block::RedstoneWire { wire: new_wire });
					update_surrounding_blocks(world, pos);
					return true;
				}
			}
			false
		}
		Block::NoteBlock { note, powered, .. } => {
			let note = (note + 1) % 25;
			let instrument = self::noteblock::get_noteblock_instrument(world, pos);

			world.set_block(
				pos,
				Block::NoteBlock {
					instrument,
					note,
					powered,
				},
			);

			if self::noteblock::is_noteblock_unblocked(world, pos) {
				self::noteblock::play_note(world, pos, instrument, note);
			}

			true
		}
		_ => false,
	}
}

pub fn update_wire_neighbors(world: &mut impl World, pos: BlockPos) {
	for direction in BlockFace::values() {
		let neighbor_pos = pos.offset(direction);
		let block = world.get_block(neighbor_pos);
		update(block, world, neighbor_pos);
		for n_direction in BlockFace::values() {
			let n_neighbor_pos = neighbor_pos.offset(n_direction);
			let block = world.get_block(n_neighbor_pos);
			update(block, world, n_neighbor_pos);
		}
	}
}

pub fn update_surrounding_blocks(world: &mut impl World, pos: BlockPos) {
	for direction in BlockFace::values() {
		let neighbor_pos = pos.offset(direction);
		let block = world.get_block(neighbor_pos);
		update(block, world, neighbor_pos);

		let up_pos = neighbor_pos.offset(BlockFace::Top);
		let up_block = world.get_block(up_pos);
		update(up_block, world, up_pos);

		let down_pos = neighbor_pos.offset(BlockFace::Bottom);
		let down_block = world.get_block(down_pos);
		update(down_block, world, down_pos);
	}
}

fn diode_get_input_strength(world: &impl World, pos: BlockPos, facing: BlockDirection) -> u8 {
	let input_pos = pos.offset(facing.into());
	let input_block = world.get_block(input_pos);
	let mut power = get_redstone_power(input_block, world, pos, facing.into());
	if matches!(power, 0) {
		if let Block::RedstoneWire { wire } = input_block {
			power = wire.power;
		}
	}

	power
}

fn get_weak_power(
	block: Block,
	world: &impl World,
	pos: BlockPos,
	side: BlockFace,
	dust_power: bool,
) -> u8 {
	match block {
		Block::RedstoneWallTorch { lit: true, facing } if BlockFace::from(facing) != side => 15,
		Block::RedstoneBlock {}
		| Block::RedstoneTorch { lit: true }
		| Block::StonePressurePlate { powered: true }
		| Block::Lever {
			lever: Lever { powered: true, .. },
		}
		| Block::StoneButton {
			button: StoneButton { powered: true, .. },
		} => 15,
		Block::RedstoneRepeater { repeater }
			if BlockFace::from(repeater.facing) == side && repeater.powered =>
		{
			15
		}
		Block::RedstoneComparator { comparator } if BlockFace::from(comparator.facing) == side => {
			if let Some(BlockEntity::Comparator { output_strength }) = world.get_block_entity(pos) {
				*output_strength
			} else {
				0
			}
		}
		Block::RedstoneWire { wire } if dust_power => match side {
			BlockFace::Top => wire.power,
			BlockFace::Bottom => 0,
			_ => {
				let direction = side.direction().unwrap();
				if self::wire::get_current_side(
					self::wire::get_regulated_sides(wire, world, pos),
					!direction,
				)
				.is_none()
				{
					0
				} else {
					wire.power
				}
			}
		},
		_ => 0,
	}
}

fn get_redstone_power_no_dust(
	block: Block,
	world: &impl World,
	pos: BlockPos,
	facing: BlockFace,
) -> u8 {
	if block.is_solid() {
		get_max_strong_power(world, pos, false)
	} else {
		get_weak_power(block, world, pos, facing, false)
	}
}

fn get_max_strong_power(world: &impl World, pos: BlockPos, dust_power: bool) -> u8 {
	let mut max_power = 0;
	for side in &BlockFace::values() {
		let block = world.get_block(pos.offset(*side));
		max_power = max_power.max(get_strong_power(
			block,
			world,
			pos.offset(*side),
			*side,
			dust_power,
		));
	}

	max_power
}

fn get_strong_power(
	block: Block,
	world: &impl World,
	pos: BlockPos,
	side: BlockFace,
	dust_power: bool,
) -> u8 {
	match block {
		Block::RedstoneWallTorch { lit: true, .. } | Block::RedstoneTorch { lit: true }
			if matches!(side, BlockFace::Bottom) =>
		{
			15
		}
		Block::Lever { lever } => bool_to_signal_strength(
			match side {
				BlockFace::Top => matches!(lever.face, LeverFace::Floor),
				BlockFace::Bottom => matches!(lever.face, LeverFace::Ceiling),
				_ => {
					matches!(lever.face, LeverFace::Wall)
						&& lever.facing == side.direction().unwrap()
				}
			} && lever.powered,
		),
		Block::StoneButton { button } => bool_to_signal_strength(
			match side {
				BlockFace::Top => matches!(button.face, ButtonFace::Floor),
				BlockFace::Bottom => matches!(button.face, ButtonFace::Ceiling),
				_ => {
					matches!(button.face, ButtonFace::Wall)
						&& button.facing == side.direction().unwrap()
				}
			} && button.powered,
		),
		Block::StonePressurePlate { powered: true } if matches!(side, BlockFace::Top) => 15,
		Block::RedstoneWire { .. }
		| Block::RedstoneRepeater { .. }
		| Block::RedstoneComparator { .. } => get_weak_power(block, world, pos, side, dust_power),
		_ => 0,
	}
}
