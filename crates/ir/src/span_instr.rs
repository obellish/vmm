use core::ops::{Range, RangeInclusive};

use serde::{Deserialize, Serialize};

use super::Offset;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpanInstruction {
	Inc { value: i8, range: RangeInclusive<Offset> },
}
