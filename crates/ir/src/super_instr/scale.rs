use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	num::NonZeroU8,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ScaleAnd {
	Move,
	Fetch,
	Take,
	Set(NonZeroU8),
}

impl Display for ScaleAnd {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::Fetch => f.write_str("fetch"),
			Self::Move => f.write_str("mov"),
			Self::Take => f.write_str("take"),
			i @ Self::Set(_) => Debug::fmt(&i, f),
		}
	}
}
