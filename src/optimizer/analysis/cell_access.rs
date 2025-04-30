use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellAccess {
	pub kind: CellAccessType,
	pub instruction_index: usize,
}

impl CellAccess {
	#[must_use]
	pub const fn unused(index: usize) -> Self {
		Self::new(CellAccessType::Unused, index)
	}

	#[must_use]
	pub const fn set(value: u8, index: usize) -> Self {
		Self::new(CellAccessType::Set(value), index)
	}

	const fn new(kind: CellAccessType, index: usize) -> Self {
		Self { kind, instruction_index: index }
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellAccessType {
	#[default]
	Unused,
	NonZero,
	Set(u8),
}
