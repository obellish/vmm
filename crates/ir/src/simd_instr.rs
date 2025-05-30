use alloc::vec::Vec;
use core::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use super::Offset;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SimdInstruction {
	IncVals {
		value: i8,
		offsets: Vec<Option<Offset>>,
	},
	SetVals {
		value: Option<NonZeroU8>,
		offsets: Vec<Option<Offset>>,
	},
}

impl SimdInstruction {
	#[must_use]
	pub const fn inc_vals(value: i8, offsets: Vec<Option<Offset>>) -> Self {
		Self::IncVals { value, offsets }
	}

	#[must_use]
	pub const fn set_vals(value: u8, offsets: Vec<Option<Offset>>) -> Self {
		Self::SetVals {
			value: NonZeroU8::new(value),
			offsets,
		}
	}
}
