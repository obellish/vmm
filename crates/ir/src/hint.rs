use core::num::NonZero;

use serde::{Deserialize, Serialize};

use super::Offset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CompilerHint {
	KnownValue {
		value: Option<NonZero<u8>>,
		offset: Offset,
	},
}

impl CompilerHint {
	#[must_use]
	pub const fn known_value(value: u8) -> Self {
		Self::KnownValue {
			value: NonZero::new(value),
			offset: Offset(0),
		}
	}

	pub fn known_value_at(value: u8, offset: impl Into<Offset>) -> Self {
		Self::KnownValue {
			value: NonZero::new(value),
			offset: offset.into(),
		}
	}
}
