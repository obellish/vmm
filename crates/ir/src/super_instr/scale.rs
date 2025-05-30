use core::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ScaleAnd {
	Move,
	Fetch,
	Take,
}

impl Display for ScaleAnd {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::Fetch => f.write_str("fetch"),
			Self::Move => f.write_str("mov"),
			Self::Take => f.write_str("take"),
		}
	}
}
