use core::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IntegerType {
	Int32,
}

impl IntegerType {
	#[must_use]
	pub const fn size(self) -> usize {
		match self {
			Self::Int32 => 4,
		}
	}
}

impl Display for IntegerType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Int32 => f.write_str("i32"),
		}
	}
}
