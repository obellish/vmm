mod ops;

use serde::{Deserialize, Serialize};

use super::TAPE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TapePointer(usize);

impl TapePointer {
	#[must_use]
	pub const fn new(value: usize) -> Option<Self> {
		if value >= TAPE_SIZE {
			None
		} else {
			Some(unsafe { Self::new_unchecked(value) })
		}
	}

	#[must_use]
	pub const fn zero() -> Self {
		unsafe { Self::new_unchecked(0) }
	}

	#[must_use]
	pub const unsafe fn new_unchecked(value: usize) -> Self {
		Self(value)
	}

	#[must_use]
	pub const fn value(self) -> usize {
		self.0
	}

	pub const fn set(&mut self, mut value: usize) {
		while value >= TAPE_SIZE {
			value -= TAPE_SIZE;
		}

		unsafe { self.set_unchecked(value) };
	}

	pub const unsafe fn set_unchecked(&mut self, value: usize) {
		self.0 = value;
	}
}

impl Default for TapePointer {
	fn default() -> Self {
		Self::zero()
	}
}
