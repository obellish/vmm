use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::Offset;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SimdInstruction {
	IncBy { value: i8, offsets: Vec<Offset> },
}

impl SimdInstruction {
	#[must_use]
	pub const fn inc_by(value: i8, offsets: Vec<Offset>) -> Self {
		Self::IncBy { value, offsets }
	}
}
