use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::Not,
	str::FromStr,
};

use super::{Block, BlockTransform, FlipDirection};
use crate::{BlockDirection, BlockProperty};

#[derive(Debug)]
#[repr(transparent)]
pub struct TryParseError(String);

impl Display for TryParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid value '")?;
		f.write_str(&self.0)?;
		f.write_char('\'')
	}
}

impl StdError for TryParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockProperty, BlockTransform)]
pub struct RedstoneRepeater {
	pub delay: u8,
	pub facing: BlockDirection,
	pub locked: bool,
	pub powered: bool,
}

impl RedstoneRepeater {
	pub(super) const fn new(
		delay: u8,
		facing: BlockDirection,
		locked: bool,
		powered: bool,
	) -> Self {
		Self {
			delay,
			facing,
			locked,
			powered,
		}
	}
}

impl Default for RedstoneRepeater {
	fn default() -> Self {
		Self::new(1, BlockDirection::default(), false, false)
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, BlockProperty, BlockTransform)]
pub struct RedstoneComparator {
	pub facing: BlockDirection,
	pub mode: ComparatorMode,
	pub powered: bool,
}

impl RedstoneComparator {
	#[must_use]
	pub const fn new(facing: BlockDirection, mode: ComparatorMode, powered: bool) -> Self {
		Self {
			facing,
			mode,
			powered,
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, BlockProperty, BlockTransform)]
pub struct Lever {
	pub face: LeverFace,
	pub facing: BlockDirection,
	pub powered: bool,
}

impl Lever {
	#[must_use]
	pub const fn new(face: LeverFace, facing: BlockDirection, powered: bool) -> Self {
		Self {
			face,
			facing,
			powered,
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, BlockProperty, BlockTransform)]
pub struct StoneButton {
	pub face: ButtonFace,
	pub facing: BlockDirection,
	pub powered: bool,
}

impl StoneButton {
	#[must_use]
	pub const fn new(face: ButtonFace, facing: BlockDirection, powered: bool) -> Self {
		Self {
			face,
			facing,
			powered,
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, BlockProperty)]
pub struct RedstoneWire {
	pub north: RedstoneWireSide,
	pub south: RedstoneWireSide,
	pub east: RedstoneWireSide,
	pub west: RedstoneWireSide,
	pub power: u8,
}

impl RedstoneWire {
	#[must_use]
	pub const fn new(
		north: RedstoneWireSide,
		south: RedstoneWireSide,
		east: RedstoneWireSide,
		west: RedstoneWireSide,
		power: u8,
	) -> Self {
		Self {
			north,
			south,
			east,
			west,
			power,
		}
	}
}

impl BlockTransform for RedstoneWire {
	fn rotate90(&mut self) {
		*self = Self {
			north: self.west,
			east: self.north,
			south: self.east,
			west: self.south,
			..*self
		}
	}

	fn flip(&mut self, dir: FlipDirection) {
		*self = match dir {
			FlipDirection::FlipX => Self {
				east: self.west,
				west: self.east,
				..*self
			},
			FlipDirection::FlipZ => Self {
				north: self.south,
				south: self.north,
				..*self
			},
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ComparatorMode {
	#[default]
	Compare,
	Subtract,
}

impl ComparatorMode {
	pub(super) const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Compare,
			1 => Self::Subtract,
			_ => return None,
		})
	}

	pub(super) const fn id(self) -> u32 {
		self as u32
	}

	#[must_use]
	pub const fn toggle(self) -> Self {
		match self {
			Self::Compare => Self::Subtract,
			Self::Subtract => Self::Compare,
		}
	}
}

impl Display for ComparatorMode {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Compare => "compare",
			Self::Subtract => "subtract",
		})
	}
}

impl FromStr for ComparatorMode {
	type Err = TryParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"subtract" => Self::Subtract,
			"compare" => Self::Compare,
			s => return Err(TryParseError(s.to_owned())),
		})
	}
}

impl Not for ComparatorMode {
	type Output = Self;

	fn not(self) -> Self::Output {
		self.toggle()
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LeverFace {
	Floor,
	#[default]
	Wall,
	Ceiling,
}

impl LeverFace {
	pub(super) const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Floor,
			1 => Self::Wall,
			2 => Self::Ceiling,
			_ => return None,
		})
	}

	pub(super) const fn id(self) -> u32 {
		self as u32
	}
}

impl Display for LeverFace {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Floor => "floor",
			Self::Wall => "wall",
			Self::Ceiling => "ceiling",
		})
	}
}

impl FromStr for LeverFace {
	type Err = TryParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"floor" => Self::Floor,
			"ceiling" => Self::Ceiling,
			"wall" => Self::Wall,
			s => return Err(TryParseError(s.to_owned())),
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ButtonFace {
	Floor,
	#[default]
	Wall,
	Ceiling,
}

impl ButtonFace {
	pub(super) const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Floor,
			1 => Self::Wall,
			2 => Self::Ceiling,
			_ => return None,
		})
	}

	pub(super) const fn id(self) -> u32 {
		self as u32
	}
}

impl Display for ButtonFace {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Floor => "floor",
			Self::Wall => "wall",
			Self::Ceiling => "ceiling",
		})
	}
}

impl FromStr for ButtonFace {
	type Err = TryParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"floor" => Self::Floor,
			"ceiling" => Self::Ceiling,
			"wall" => Self::Wall,
			s => return Err(TryParseError(s.to_owned())),
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RedstoneWireSide {
	Up,
	Side,
	#[default]
	None,
}

impl RedstoneWireSide {
	#[must_use]
	pub const fn is_none(self) -> bool {
		matches!(self, Self::None)
	}

	pub(super) const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Up,
			1 => Self::Side,
			2 => Self::None,
			_ => return None,
		})
	}

	pub(super) const fn id(self) -> u32 {
		self as u32
	}
}

impl Display for RedstoneWireSide {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Up => "up",
			Self::Side => "side",
			Self::None => "none",
		})
	}
}

impl FromStr for RedstoneWireSide {
	type Err = TryParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"up" => Self::Up,
			"side" => Self::Side,
			"none" => Self::None,
			s => return Err(TryParseError(s.to_owned())),
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TrapdoorHalf {
	#[default]
	Top,
	Bottom,
}

impl TrapdoorHalf {
	pub(super) const fn id(self) -> u32 {
		self as u32
	}

	pub(super) const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Top,
			1 => Self::Bottom,
			_ => return None,
		})
	}
}

impl Display for TrapdoorHalf {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Top => "top",
			Self::Bottom => "bottom",
		})
	}
}

impl FromStr for TrapdoorHalf {
	type Err = TryParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"top" => Self::Top,
			"bottom" => Self::Bottom,
			s => return Err(TryParseError(s.to_owned())),
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Instrument {
	Harp,
	Basedrum,
	Snare,
	Hat,
	Bass,
	Flute,
	Bell,
	Guitar,
	Chime,
	Xylophone,
	IronXylophone,
	CowBell,
	Didgeridoo,
	Bit,
	Banjo,
	Pling,
	Zombie,
	Skeleton,
	Creeper,
	Dragon,
	WitherSkeleton,
	Piglin,
}

impl Instrument {
	pub(super) const fn id(self) -> u32 {
		self as u32
	}

	pub(super) const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Harp,
			1 => Self::Basedrum,
			2 => Self::Snare,
			3 => Self::Hat,
			4 => Self::Bass,
			5 => Self::Flute,
			6 => Self::Bell,
			7 => Self::Guitar,
			8 => Self::Chime,
			9 => Self::Xylophone,
			10 => Self::IronXylophone,
			11 => Self::CowBell,
			12 => Self::Didgeridoo,
			13 => Self::Bit,
			14 => Self::Banjo,
			15 => Self::Pling,
			16 => Self::Zombie,
			17 => Self::Skeleton,
			18 => Self::Creeper,
			19 => Self::Dragon,
			20 => Self::WitherSkeleton,
			21 => Self::Piglin,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn from_block_below(block: Block) -> Self {
		match block {
			Block::Stone {}
			| Block::CoalBlock {}
			| Block::Quartz {}
			| Block::Sandstone {}
			| Block::Concrete { .. }
			| Block::Terracotta {}
			| Block::ColoredTerracotta { .. } => Self::Basedrum,
			Block::Sand {} => Self::Snare,
			Block::Glass {} | Block::StainedGlass { .. } => Self::Hat,
			Block::Sign { .. }
			| Block::NoteBlock { .. }
			| Block::Barrel {}
			| Block::Composter { .. } => Self::Bass,
			Block::Clay {} => Self::Flute,
			Block::GoldBlock {} => Self::Bell,
			Block::Wool { .. } => Self::Guitar,
			Block::PackedIce {} => Self::Chime,
			Block::BoneBlock {} => Self::Xylophone,
			Block::IronBlock {} => Self::IronXylophone,
			Block::SoulSand {} => Self::CowBell,
			Block::Pumpkin {} => Self::Didgeridoo,
			Block::EmeraldBlock {} => Self::Bit,
			Block::HayBlock {} => Self::Banjo,
			Block::Glowstone { .. } => Self::Pling,
			_ => Self::Harp,
		}
	}

	#[must_use]
	pub const fn to_sound_id(self) -> i32 {
		match self {
			Self::Harp => 945,
			Self::Basedrum => 939,
			Self::Snare => 948,
			Self::Hat => 946,
			Self::Bass => 940,
			Self::Flute => 943,
			Self::Bell => 941,
			Self::Guitar => 944,
			Self::Chime => 942,
			Self::Xylophone => 949,
			Self::IronXylophone => 950,
			Self::CowBell => 951,
			Self::Didgeridoo => 952,
			Self::Bit => 953,
			Self::Banjo => 954,
			Self::Pling => 947,
			Self::Zombie => 955,
			Self::Skeleton => 956,
			Self::Creeper => 957,
			Self::Dragon => 958,
			Self::WitherSkeleton => 959,
			Self::Piglin => 960,
		}
	}
}

impl Display for Instrument {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Harp => "harp",
			Self::Basedrum => "basedrum",
			Self::Snare => "snare",
			Self::Hat => "hat",
			Self::Bass => "bass",
			Self::Flute => "flute",
			Self::Bell => "bell",
			Self::Guitar => "guitar",
			Self::Chime => "chime",
			Self::Xylophone => "xylophone",
			Self::IronXylophone => "iron_xylophone",
			Self::CowBell => "cow_bell",
			Self::Didgeridoo => "didgeridoo",
			Self::Bit => "bit",
			Self::Banjo => "banjo",
			Self::Pling => "pling",
			Self::Zombie => "zombie",
			Self::Skeleton => "skeleton",
			Self::Creeper => "creeper",
			Self::Dragon => "dragon",
			Self::WitherSkeleton => "wither_skeleton",
			Self::Piglin => "piglin",
		})
	}
}

impl FromStr for Instrument {
	type Err = TryParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"harp" => Self::Harp,
			"basedrum" => Self::Basedrum,
			"snare" => Self::Snare,
			"hat" => Self::Hat,
			"bass" => Self::Bass,
			"flute" => Self::Flute,
			"bell" => Self::Bell,
			"guitar" => Self::Guitar,
			"chime" => Self::Chime,
			"xylophone" => Self::Xylophone,
			"iron_xylophone" => Self::IronXylophone,
			"cow_bell" => Self::CowBell,
			"didgeridoo" => Self::Didgeridoo,
			"bit" => Self::Bit,
			"banjo" => Self::Banjo,
			"pling" => Self::Pling,
			s => return Err(TryParseError(s.to_owned())),
		})
	}
}
