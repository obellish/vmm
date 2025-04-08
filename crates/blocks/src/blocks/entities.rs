use std::{
	collections::HashMap,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryEntry {
	pub id: u32,
	pub slot: i8,
	pub count: i8,
	pub nbt: Option<Vec<u8>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SignBlockEntity {
	pub front_rows: [String; 4],
	pub back_rows: [String; 4],
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TryParseContainerTypeError(String);

impl Display for TryParseContainerTypeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid container type ")?;
		f.write_str(&self.0)
	}
}

impl StdError for TryParseContainerTypeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerType {
	Furnace,
	Barrel,
	Hopper,
}

impl ContainerType {
	#[must_use]
	pub const fn slots(self) -> u8 {
		match self {
			Self::Furnace => 3,
			Self::Barrel => 27,
			Self::Hopper => 5,
		}
	}

	#[must_use]
	pub const fn window_type(self) -> u8 {
		match self {
			Self::Furnace => 14,
			Self::Barrel => 2,
			Self::Hopper => 16,
		}
	}
}

impl Display for ContainerType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("minecraft:")?;
		f.write_str(match self {
			Self::Furnace => "furnace",
			Self::Barrel => "barrel",
			Self::Hopper => "hopper",
		})
	}
}

impl FromStr for ContainerType {
	type Err = TryParseContainerTypeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"barrel" => Self::Barrel,
			"furnace" => Self::Furnace,
			"hopper" => Self::Hopper,
			s => return Err(TryParseContainerTypeError(s.to_owned())),
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockEntity {
	Comparator {
		output_strength: u8,
	},
	Container {
		comparator_override: u8,
		inventory: Vec<InventoryEntry>,
		ty: ContainerType,
	},
	Sign(SignBlockEntity),
}

impl BlockEntity {
	#[must_use]
	pub const fn ty(&self) -> i32 {
		match self {
			Self::Comparator { .. } => 18,
			Self::Sign(_) => 7,
			Self::Container { ty, .. } => match ty {
				ContainerType::Furnace => 0,
				ContainerType::Barrel => 26,
				ContainerType::Hopper => 17,
			},
		}
	}
}
