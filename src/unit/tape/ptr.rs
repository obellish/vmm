use std::ops::{Add, AddAssign, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use super::TAPE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TapePointer(usize);

impl TapePointer {
	#[must_use]
	pub const fn new() -> Self {
		Self(0)
	}

	#[must_use]
	pub const fn value(self) -> usize {
		self.0
	}

	pub const fn set(&mut self, mut value: usize) {
		while value >= TAPE_SIZE {
			value -= TAPE_SIZE;
		}

		self.0 = value;
	}
}

impl Default for TapePointer {
	fn default() -> Self {
		Self::new()
	}
}

impl Add<usize> for TapePointer {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		let mut out = Self(self.0 + rhs);

		if out.0 >= TAPE_SIZE {
			out.0 -= TAPE_SIZE;
		}

		out
	}
}

impl AddAssign<usize> for TapePointer {
	fn add_assign(&mut self, rhs: usize) {
		*self = *self + rhs;
	}
}

impl Sub<usize> for TapePointer {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Self(if self.0.wrapping_sub(rhs) >= TAPE_SIZE {
			TAPE_SIZE - rhs
		} else {
			self.0 - rhs
		})
	}
}

impl SubAssign<usize> for TapePointer {
	fn sub_assign(&mut self, rhs: usize) {
		*self = *self - rhs;
	}
}
