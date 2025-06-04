use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};
use vmm_wrap::ops::{WrappingAdd, WrappingAddAssign, WrappingNeg, WrappingSub, WrappingSubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Offset {
	Relative(isize),
	Absolute(usize),
}

impl Offset {
	#[must_use]
	pub const fn abs(self) -> Self {
		match self {
			Self::Relative(i) => Self::Relative(i.abs()),
			Self::Absolute(a) => Self::Absolute(a),
		}
	}

	#[must_use]
	pub const fn is_relative(self) -> bool {
		matches!(self, Self::Relative(_))
	}

	#[must_use]
	pub const fn is_absolute(self) -> bool {
		matches!(self, Self::Absolute(_))
	}
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let alt = f.alternate();

		match self {
			Self::Relative(offset) => {
				if alt {
					f.write_char('[')?;
				}

				Display::fmt(&offset, f)?;

				if alt {
					f.write_char(']')?;
				}
			}
			Self::Absolute(offset) => {
				if alt {
					f.write_char('{')?;
				}

				Display::fmt(&offset, f)?;

				if alt {
					f.write_char('}')?;
				}
			}
		}

		Ok(())
	}
}

impl From<isize> for Offset {
	fn from(value: isize) -> Self {
		Self::Relative(value)
	}
}

impl From<&isize> for Offset {
	fn from(value: &isize) -> Self {
		(*value).into()
	}
}

impl From<usize> for Offset {
	fn from(value: usize) -> Self {
		Self::Absolute(value)
	}
}

impl From<&usize> for Offset {
	fn from(value: &usize) -> Self {
		(*value).into()
	}
}

impl WrappingAdd for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(l), Self::Relative(r)) => {
				Self::Relative(WrappingAdd::wrapping_add(l, r))
			}
			(Self::Absolute(l), Self::Absolute(r)) => {
				Self::Absolute(WrappingAdd::wrapping_add(l, r))
			}
			(Self::Absolute(l), Self::Relative(r)) => {
				Self::Absolute(WrappingAdd::wrapping_add(l, r))
			}
			(Self::Relative(l), Self::Absolute(r)) => {
				Self::Relative(WrappingAdd::wrapping_add(l, r))
			}
		}
	}
}

impl WrappingAdd<Offset> for &Offset {
	type Output = <Offset as WrappingAdd>::Output;

	fn wrapping_add(self, rhs: Offset) -> Self::Output {
		(*self).wrapping_add(rhs)
	}
}

impl WrappingAdd<&Self> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: &Self) -> Self::Output {
		self.wrapping_add(*rhs)
	}
}

impl WrappingAdd for &Offset {
	type Output = <Offset as WrappingAdd>::Output;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		(*self).wrapping_add(*rhs)
	}
}

impl WrappingAdd<isize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: isize) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(WrappingAdd::wrapping_add(a, rhs)),
			Self::Relative(r) => Self::Relative(WrappingAdd::wrapping_add(r, rhs)),
		}
	}
}

impl WrappingAdd<isize> for &Offset {
	type Output = <Offset as WrappingAdd<isize>>::Output;

	fn wrapping_add(self, rhs: isize) -> Self::Output {
		(*self).wrapping_add(rhs)
	}
}

impl WrappingAdd<&isize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: &isize) -> Self::Output {
		self.wrapping_add(*rhs)
	}
}

impl WrappingAdd<&isize> for &Offset {
	type Output = <Offset as WrappingAdd<isize>>::Output;

	fn wrapping_add(self, rhs: &isize) -> Self::Output {
		(*self).wrapping_add(*rhs)
	}
}

impl WrappingAdd<usize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: usize) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(WrappingAdd::wrapping_add(a, rhs)),
			Self::Relative(r) => Self::Relative(WrappingAdd::wrapping_add(r, rhs)),
		}
	}
}

impl WrappingAdd<usize> for &Offset {
	type Output = <Offset as WrappingAdd<usize>>::Output;

	fn wrapping_add(self, rhs: usize) -> Self::Output {
		(*self).wrapping_add(rhs)
	}
}

impl WrappingAdd<&usize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: &usize) -> Self::Output {
		self.wrapping_add(*rhs)
	}
}

impl WrappingAdd<&usize> for &Offset {
	type Output = <Offset as WrappingAdd<usize>>::Output;

	fn wrapping_add(self, rhs: &usize) -> Self::Output {
		(*self).wrapping_add(*rhs)
	}
}

impl WrappingAddAssign for Offset {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<&Self> for Offset {
	fn wrapping_add_assign(&mut self, rhs: &Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<isize> for Offset {
	fn wrapping_add_assign(&mut self, rhs: isize) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<&isize> for Offset {
	fn wrapping_add_assign(&mut self, rhs: &isize) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<usize> for Offset {
	fn wrapping_add_assign(&mut self, rhs: usize) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<&usize> for Offset {
	fn wrapping_add_assign(&mut self, rhs: &usize) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingNeg for Offset {
	type Output = Self;

	#[inline]
	fn wrapping_neg(self) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(a.wrapping_neg()),
			Self::Relative(r) => Self::Relative(r.wrapping_neg()),
		}
	}
}

impl WrappingNeg for &Offset {
	type Output = Offset;

	fn wrapping_neg(self) -> Self::Output {
		(*self).wrapping_neg()
	}
}

impl WrappingSub for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(l), Self::Relative(r)) => {
				Self::Relative(WrappingSub::wrapping_sub(l, r))
			}
			(Self::Absolute(l), Self::Absolute(r)) => {
				Self::Absolute(WrappingSub::wrapping_sub(l, r))
			}
			(Self::Relative(l), Self::Absolute(r)) => {
				Self::Relative(WrappingSub::wrapping_sub(l, r))
			}
			(Self::Absolute(l), Self::Relative(r)) => {
				Self::Absolute(WrappingSub::wrapping_sub(l, r))
			}
		}
	}
}

impl WrappingSub<Offset> for &Offset {
	type Output = <Offset as WrappingSub>::Output;

	fn wrapping_sub(self, rhs: Offset) -> Self::Output {
		(*self).wrapping_sub(rhs)
	}
}

impl WrappingSub<&Self> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: &Self) -> Self::Output {
		self.wrapping_sub(*rhs)
	}
}

impl WrappingSub for &Offset {
	type Output = <Offset as WrappingSub>::Output;

	fn wrapping_sub(self, rhs: Self) -> Self::Output {
		(*self).wrapping_sub(*rhs)
	}
}

impl WrappingSub<isize> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: isize) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(WrappingSub::wrapping_sub(a, rhs)),
			Self::Relative(r) => Self::Relative(WrappingSub::wrapping_sub(r, rhs)),
		}
	}
}

impl WrappingSub<isize> for &Offset {
	type Output = <Offset as WrappingSub<isize>>::Output;

	fn wrapping_sub(self, rhs: isize) -> Self::Output {
		(*self).wrapping_sub(rhs)
	}
}

impl WrappingSub<&isize> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: &isize) -> Self::Output {
		self.wrapping_sub(*rhs)
	}
}

impl WrappingSub<&isize> for &Offset {
	type Output = <Offset as WrappingAdd<isize>>::Output;

	fn wrapping_sub(self, rhs: &isize) -> Self::Output {
		(*self).wrapping_sub(*rhs)
	}
}

impl WrappingSub<usize> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: usize) -> Self::Output {
		match self {
			Self::Absolute(a) => Self::Absolute(WrappingSub::wrapping_sub(a, rhs)),
			Self::Relative(r) => Self::Relative(WrappingSub::wrapping_sub(r, rhs)),
		}
	}
}

impl WrappingSub<usize> for &Offset {
	type Output = <Offset as WrappingSub<usize>>::Output;

	fn wrapping_sub(self, rhs: usize) -> Self::Output {
		(*self).wrapping_sub(rhs)
	}
}

impl WrappingSub<&usize> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: &usize) -> Self::Output {
		self.wrapping_sub(*rhs)
	}
}

impl WrappingSub<&usize> for &Offset {
	type Output = <Offset as WrappingSub<usize>>::Output;

	fn wrapping_sub(self, rhs: &usize) -> Self::Output {
		(*self).wrapping_sub(*rhs)
	}
}

impl WrappingSubAssign for Offset {
	fn wrapping_sub_assign(&mut self, rhs: Self) {
		*self = self.wrapping_sub(rhs);
	}
}

impl WrappingSubAssign<&Self> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: &Self) {
		*self = self.wrapping_sub(rhs);
	}
}

impl WrappingSubAssign<isize> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: isize) {
		*self = self.wrapping_sub(rhs);
	}
}

impl WrappingSubAssign<&isize> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: &isize) {
		*self = self.wrapping_sub(rhs);
	}
}

impl WrappingSubAssign<usize> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: usize) {
		*self = self.wrapping_sub(rhs);
	}
}

impl WrappingSubAssign<&usize> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: &usize) {
		*self = self.wrapping_sub(rhs);
	}
}
