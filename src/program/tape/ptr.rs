use std::ops::{Add, AddAssign, Sub, SubAssign};

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

impl Add<isize> for TapePointer {
	type Output = Self;

	fn add(self, rhs: isize) -> Self::Output {
		if rhs < 0 {
			self - rhs.unsigned_abs()
		} else {
			self + rhs.unsigned_abs()
		}
	}
}

impl AddAssign<isize> for TapePointer {
	fn add_assign(&mut self, rhs: isize) {
		*self = *self + rhs;
	}
}

impl Default for TapePointer {
	fn default() -> Self {
		unsafe { Self::new_unchecked(0) }
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

impl Sub<isize> for TapePointer {
	type Output = Self;

	fn sub(self, rhs: isize) -> Self::Output {
		self + -rhs
	}
}

impl SubAssign<isize> for TapePointer {
	fn sub_assign(&mut self, rhs: isize) {
		*self = *self - rhs;
	}
}
