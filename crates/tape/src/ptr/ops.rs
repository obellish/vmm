use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use crate::{TAPE_SIZE, TapePointer};

impl Add<usize> for TapePointer {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		let mut out = Self(self.0 + rhs);

		while out.0 >= TAPE_SIZE {
			out.0 -= TAPE_SIZE;
		}

		out
	}
}

impl Add<&usize> for TapePointer {
	type Output = Self;

	fn add(self, rhs: &usize) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<usize> for &TapePointer {
	type Output = TapePointer;

	fn add(self, rhs: usize) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add<&usize> for &TapePointer {
	type Output = TapePointer;

	fn add(self, rhs: &usize) -> Self::Output {
		Add::add(*self, *rhs)
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

impl Add<&isize> for TapePointer {
	type Output = Self;

	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<isize> for &TapePointer {
	type Output = TapePointer;

	fn add(self, rhs: isize) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add<&isize> for &TapePointer {
	type Output = TapePointer;

	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl AddAssign<usize> for TapePointer {
	fn add_assign(&mut self, rhs: usize) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&usize> for TapePointer {
	fn add_assign(&mut self, rhs: &usize) {
		AddAssign::add_assign(self, *rhs);
	}
}

impl AddAssign<isize> for TapePointer {
	fn add_assign(&mut self, rhs: isize) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&isize> for TapePointer {
	fn add_assign(&mut self, rhs: &isize) {
		AddAssign::add_assign(self, *rhs);
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

impl Sub<&usize> for TapePointer {
	type Output = Self;

	fn sub(self, rhs: &usize) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<usize> for &TapePointer {
	type Output = TapePointer;

	fn sub(self, rhs: usize) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub<&usize> for &TapePointer {
	type Output = TapePointer;

	fn sub(self, rhs: &usize) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}

impl Sub<isize> for TapePointer {
	type Output = Self;

	fn sub(self, rhs: isize) -> Self::Output {
		Self::add(self, Neg::neg(rhs))
	}
}

impl Sub<&isize> for TapePointer {
	type Output = Self;

	fn sub(self, rhs: &isize) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<isize> for &TapePointer {
	type Output = TapePointer;

	fn sub(self, rhs: isize) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub<&isize> for &TapePointer {
	type Output = TapePointer;

	fn sub(self, rhs: &isize) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}

impl SubAssign<usize> for TapePointer {
	fn sub_assign(&mut self, rhs: usize) {
		*self = Sub::sub(*self, rhs);
	}
}

impl SubAssign<&usize> for TapePointer {
	fn sub_assign(&mut self, rhs: &usize) {
		SubAssign::sub_assign(self, *rhs);
	}
}

impl SubAssign<isize> for TapePointer {
	fn sub_assign(&mut self, rhs: isize) {
		*self = Sub::sub(*self, rhs);
	}
}

impl SubAssign<&isize> for TapePointer {
	fn sub_assign(&mut self, rhs: &isize) {
		SubAssign::sub_assign(self, *rhs);
	}
}
