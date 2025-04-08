#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

extern crate self as vmm_blocks;

pub mod blocks;
pub mod items;

use std::{
	collections::HashMap,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::{Add, Mul, Not, Sub},
	str::FromStr,
};

use serde::{Deserialize, Serialize};
pub use vmm_derive::BlockProperty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BlockPos {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl BlockPos {
	#[must_use]
	pub const fn new(x: i32, y: i32, z: i32) -> Self {
		Self { x, y, z }
	}

	#[must_use]
	pub const fn zero() -> Self {
		Self::new(0, 0, 0)
	}

	#[must_use]
	pub const fn offset(self, face: BlockFace) -> Self {
		match face {
			BlockFace::Bottom => Self::new(self.x, self.y.saturating_sub(1), self.z),
			BlockFace::Top => Self::new(self.x, self.y + 1, self.z),
			BlockFace::North => Self::new(self.x, self.y, self.z - 1),
			BlockFace::South => Self::new(self.x, self.y, self.z + 1),
			BlockFace::West => Self::new(self.x - 1, self.y, self.z),
			BlockFace::East => Self::new(self.x + 1, self.y, self.z),
		}
	}
}

impl Add for BlockPos {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
	}
}

impl Display for BlockPos {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('(')?;
		Display::fmt(&self.x, f)?;
		f.write_str(", ")?;
		Display::fmt(&self.y, f)?;
		f.write_str(", ")?;
		Display::fmt(&self.z, f)?;
		f.write_char(')')
	}
}

impl Mul<i32> for BlockPos {
	type Output = Self;

	fn mul(self, rhs: i32) -> Self::Output {
		Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
	}
}

impl Sub for BlockPos {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SignType(pub u32);

impl SignType {
	#[must_use]
	pub const fn from_sign_type(sign_type: u32) -> Option<Self> {
		Some(Self(match sign_type {
			n @ 0..=7 => n,
			_ => return None,
		}))
	}
}

impl BlockProperty for SignType {
	fn decode(&mut self, _: &str, _: &HashMap<&str, &str>) {}

	fn encode(self, _: &'static str, _: &mut HashMap<&'static str, String>) {}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TryParseCardinalDirectionError(String);

impl Display for TryParseCardinalDirectionError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid direction ")?;
		f.write_str(&self.0)
	}
}

impl StdError for TryParseCardinalDirectionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BlockFace {
	Bottom,
	Top,
	North,
	South,
	West,
	East,
}

impl BlockFace {
	#[must_use]
	pub const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::Bottom,
			1 => Self::Top,
			2 => Self::North,
			3 => Self::South,
			4 => Self::West,
			5 => Self::East,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn values() -> [Self; 6] {
		[
			Self::Top,
			Self::Bottom,
			Self::North,
			Self::South,
			Self::East,
			Self::West,
		]
	}

	#[must_use]
	pub const fn is_horizontal(self) -> bool {
		matches!(self, Self::North | Self::South | Self::East | Self::West)
	}

	#[must_use]
	pub const fn direction(self) -> Option<BlockDirection> {
		Some(match self {
			Self::North => BlockDirection::North,
			Self::South => BlockDirection::South,
			Self::East => BlockDirection::East,
			Self::West => BlockDirection::West,
			_ => return None,
		})
	}
}

impl From<BlockDirection> for BlockFace {
	fn from(value: BlockDirection) -> Self {
		match value {
			BlockDirection::East => Self::East,
			BlockDirection::North => Self::North,
			BlockDirection::South => Self::South,
			BlockDirection::West => Self::West,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BlockColorVariant {
	White = 0,
	Orange = 1,
	Magenta = 2,
	LightBlue = 3,
	Yellow = 4,
	Lime = 5,
	Pink = 6,
	Gray = 7,
	LightGray = 8,
	Cyan = 9,
	Purple = 10,
	Blue = 11,
	Brown = 12,
	Green = 13,
	Red = 14,
	Black = 15,
}

impl BlockColorVariant {
	#[must_use]
	pub const fn id(self) -> u32 {
		self as u32
	}

	#[must_use]
	pub const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::White,
			1 => Self::Orange,
			2 => Self::Magenta,
			3 => Self::LightBlue,
			4 => Self::Yellow,
			5 => Self::Lime,
			6 => Self::Pink,
			7 => Self::Gray,
			8 => Self::LightGray,
			9 => Self::Cyan,
			10 => Self::Purple,
			11 => Self::Blue,
			12 => Self::Brown,
			13 => Self::Green,
			14 => Self::Red,
			15 => Self::Black,
			_ => return None,
		})
	}
}

impl BlockProperty for BlockColorVariant {
	fn decode(&mut self, _: &str, _: &HashMap<&str, &str>) {}

	fn encode(self, _: &'static str, _: &mut HashMap<&'static str, String>) {}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BlockDirection {
	North,
	South,
	#[default]
	West,
	East,
}

impl BlockDirection {
	#[must_use]
	pub const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::North,
			1 => Self::South,
			2 => Self::West,
			3 => Self::East,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn id(self) -> u32 {
		self as u32
	}

	#[must_use]
	pub const fn rotate_cw(self) -> Self {
		match self {
			Self::North => Self::East,
			Self::East => Self::South,
			Self::South => Self::West,
			Self::West => Self::North,
		}
	}

	#[must_use]
	pub const fn rotate_ccw(self) -> Self {
		match self {
			Self::North => Self::West,
			Self::West => Self::South,
			Self::South => Self::East,
			Self::East => Self::North,
		}
	}

	#[must_use]
	pub const fn from_rotation(rotation: u32) -> Option<Self> {
		Some(match rotation {
			0 => Self::South,
			4 => Self::West,
			8 => Self::North,
			12 => Self::East,
			_ => return None,
		})
	}
}

impl Display for BlockDirection {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::North => "north",
			Self::South => "south",
			Self::East => "east",
			Self::West => "west",
		})
	}
}

impl FromStr for BlockDirection {
	type Err = TryParseCardinalDirectionError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"north" => Self::North,
			"south" => Self::South,
			"east" => Self::East,
			"west" => Self::West,
			s => return Err(TryParseCardinalDirectionError(s.to_owned())),
		})
	}
}

impl Not for BlockDirection {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::North => Self::South,
			Self::South => Self::North,
			Self::East => Self::West,
			Self::West => Self::East,
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BlockFacing {
	North,
	East,
	South,
	#[default]
	West,
	Up,
	Down,
}

impl BlockFacing {
	#[must_use]
	pub const fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Self::North,
			1 => Self::East,
			2 => Self::South,
			3 => Self::West,
			4 => Self::Up,
			5 => Self::Down,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn id(self) -> u32 {
		self as u32
	}

	#[must_use]
	pub const fn offset_pos(self, mut pos: BlockPos, n: i32) -> BlockPos {
		match self {
			Self::North => pos.z -= n,
			Self::South => pos.z += n,
			Self::East => pos.x += n,
			Self::West => pos.x -= n,
			Self::Up => pos.y += n,
			Self::Down => pos.y -= n,
		}

		pos
	}

	#[must_use]
	pub const fn rotate_cw(self) -> Self {
		match self {
			Self::North => Self::East,
			Self::East => Self::South,
			Self::South => Self::West,
			Self::West => Self::North,
			other => other,
		}
	}

	#[must_use]
	pub const fn rotate_ccw(self) -> Self {
		match self {
			Self::North => Self::West,
			Self::West => Self::South,
			Self::South => Self::East,
			Self::East => Self::North,
			other => other,
		}
	}
}

impl Display for BlockFacing {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::North => "north",
			Self::South => "south",
			Self::East => "east",
			Self::West => "west",
			Self::Up => "up",
			Self::Down => "down",
		})
	}
}

impl From<BlockDirection> for BlockFacing {
	fn from(value: BlockDirection) -> Self {
		match value {
			BlockDirection::East => Self::East,
			BlockDirection::North => Self::North,
			BlockDirection::South => Self::South,
			BlockDirection::West => Self::West,
		}
	}
}

impl FromStr for BlockFacing {
	type Err = TryParseCardinalDirectionError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"north" => Self::North,
			"south" => Self::South,
			"east" => Self::East,
			"west" => Self::West,
			"up" => Self::Up,
			"down" => Self::Down,
			s => return Err(TryParseCardinalDirectionError(s.to_owned())),
		})
	}
}

impl Not for BlockFacing {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::Down => Self::Up,
			Self::Up => Self::Down,
			Self::North => Self::South,
			Self::South => Self::North,
			Self::East => Self::West,
			Self::West => Self::East,
		}
	}
}

pub trait BlockProperty: Sized {
	fn encode(self, name: &'static str, props: &mut HashMap<&'static str, String>);

	fn decode(&mut self, name: &str, props: &HashMap<&str, &str>);
}

impl<T> BlockProperty for T
where
	T: FromStr + ToString,
{
	fn encode(self, name: &'static str, props: &mut HashMap<&'static str, String>) {
		props.insert(name, self.to_string());
	}

	fn decode(&mut self, name: &str, props: &HashMap<&str, &str>) {
		if let Some(&s) = props.get(name) {
			if let Ok(val) = s.parse() {
				*self = val;
			}
		}
	}
}
