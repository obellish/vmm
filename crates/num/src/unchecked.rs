use core::{
	cmp::Ordering,
	fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult, UpperHex},
	ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ops::{
	UncheckedAdd, UncheckedAddAssign, UncheckedMul, UncheckedMulAssign, UncheckedSub,
	UncheckedSubAssign,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Unchecked<T>(pub T);

impl<T> Unchecked<T> {
	pub unsafe fn add<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: UncheckedAdd<Rhs>,
	{
		Add::add(Self(lhs), rhs).0
	}

	pub unsafe fn mul<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: UncheckedMul<Rhs>,
	{
		Mul::mul(Self(lhs), rhs).0
	}

	pub unsafe fn sub<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: UncheckedSub<Rhs>,
	{
		Sub::sub(Self(lhs), rhs).0
	}
}

impl<T, Rhs> Add<Rhs> for Unchecked<T>
where
	T: UncheckedAdd<Rhs>,
{
	type Output = Unchecked<T::Output>;

	fn add(self, rhs: Rhs) -> Self::Output {
		Unchecked(unsafe { UncheckedAdd::unchecked_add(self.0, rhs) })
	}
}

impl<T, Rhs> AddAssign<Rhs> for Unchecked<T>
where
	T: UncheckedAddAssign<Rhs>,
{
	fn add_assign(&mut self, rhs: Rhs) {
		unsafe {
			UncheckedAddAssign::unchecked_add_assign(&mut self.0, rhs);
		}
	}
}

#[cfg(feature = "arbitrary")]
impl<'a, T> arbitrary::Arbitrary<'a> for Unchecked<T>
where
	T: arbitrary::Arbitrary<'a>,
{
	fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
		T::arbitrary(u).map(Self)
	}

	fn arbitrary_take_rest(u: arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
		T::arbitrary_take_rest(u).map(Self)
	}

	fn size_hint(depth: usize) -> (usize, Option<usize>) {
		T::size_hint(depth)
	}

	fn try_size_hint(
		depth: usize,
	) -> arbitrary::Result<(usize, Option<usize>), arbitrary::MaxRecursionReached> {
		T::try_size_hint(depth)
	}
}

impl<T: Binary> Binary for Unchecked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Binary::fmt(&self.0, f)
	}
}

impl<T: Debug> Debug for Unchecked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'de, T> Deserialize<'de> for Unchecked<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		T::deserialize(deserializer).map(Self)
	}
}

impl<T: Display> Display for Unchecked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<T> From<T> for Unchecked<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<T: LowerHex> LowerHex for Unchecked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		LowerHex::fmt(&self.0, f)
	}
}

impl<T, Rhs> Mul<Rhs> for Unchecked<T>
where
	T: UncheckedMul<Rhs>,
{
	type Output = Unchecked<T::Output>;

	fn mul(self, rhs: Rhs) -> Self::Output {
		Unchecked(unsafe { UncheckedMul::unchecked_mul(self.0, rhs) })
	}
}

impl<T, Rhs> MulAssign<Rhs> for Unchecked<T>
where
	T: UncheckedMulAssign<Rhs>,
{
	fn mul_assign(&mut self, rhs: Rhs) {
		unsafe {
			UncheckedMulAssign::unchecked_mul_assign(&mut self.0, rhs);
		}
	}
}

impl<T: Octal> Octal for Unchecked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Octal::fmt(&self.0, f)
	}
}

impl<T: PartialEq> PartialEq<T> for Unchecked<T> {
	fn eq(&self, other: &T) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl<T: PartialOrd> PartialOrd<T> for Unchecked<T> {
	fn partial_cmp(&self, other: &T) -> Option<Ordering> {
		PartialOrd::partial_cmp(&self.0, other)
	}
}

impl<T: Serialize> Serialize for Unchecked<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Serialize::serialize(&self.0, serializer)
	}
}

impl<T, Rhs> Sub<Rhs> for Unchecked<T>
where
	T: UncheckedSub<Rhs>,
{
	type Output = Unchecked<T::Output>;

	fn sub(self, rhs: Rhs) -> Self::Output {
		Unchecked(unsafe { UncheckedSub::unchecked_sub(self.0, rhs) })
	}
}

impl<T, Rhs> SubAssign<Rhs> for Unchecked<T>
where
	T: UncheckedSubAssign<Rhs>,
{
	fn sub_assign(&mut self, rhs: Rhs) {
		unsafe {
			UncheckedSubAssign::unchecked_sub_assign(&mut self.0, rhs);
		}
	}
}

unsafe impl<T, Rhs> UncheckedAdd<Rhs> for Unchecked<T>
where
	T: UncheckedAdd<Rhs>,
{
	type Output = Unchecked<T::Output>;

	unsafe fn unchecked_add(self, rhs: Rhs) -> Self::Output {
		Add::add(self, rhs)
	}
}

unsafe impl<T, Rhs> UncheckedAddAssign<Rhs> for Unchecked<T>
where
	T: UncheckedAddAssign<Rhs>,
{
	unsafe fn unchecked_add_assign(&mut self, rhs: Rhs) {
		AddAssign::add_assign(self, rhs);
	}
}

unsafe impl<T, Rhs> UncheckedMul<Rhs> for Unchecked<T>
where
	T: UncheckedMul<Rhs>,
{
	type Output = Unchecked<T::Output>;

	unsafe fn unchecked_mul(self, rhs: Rhs) -> Self::Output {
		Mul::mul(self, rhs)
	}
}

unsafe impl<T, Rhs> UncheckedMulAssign<Rhs> for Unchecked<T>
where
	T: UncheckedMulAssign<Rhs>,
{
	unsafe fn unchecked_mul_assign(&mut self, rhs: Rhs) {
		MulAssign::mul_assign(self, rhs);
	}
}

unsafe impl<T, Rhs> UncheckedSub<Rhs> for Unchecked<T>
where
	T: UncheckedSub<Rhs>,
{
	type Output = Unchecked<T::Output>;

	unsafe fn unchecked_sub(self, rhs: Rhs) -> Self::Output {
		Sub::sub(self, rhs)
	}
}

unsafe impl<T, Rhs> UncheckedSubAssign<Rhs> for Unchecked<T>
where
	T: UncheckedSubAssign<Rhs>,
{
	unsafe fn unchecked_sub_assign(&mut self, rhs: Rhs) {
		SubAssign::sub_assign(self, rhs);
	}
}

impl<T: UpperHex> UpperHex for Unchecked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		UpperHex::fmt(&self.0, f)
	}
}

#[cfg(test)]
mod tests {
	use core::fmt::Debug;

	use crate::{Unchecked, ops::UncheckedAdd};

	fn check_add<T>(value: T, one: T, expected: T)
	where
		T: Debug + PartialEq + UncheckedAdd<T, Output = T>,
	{
		assert_eq!(unsafe { Unchecked::add(value, one) }, expected);
	}

	#[test]
	fn add() {
		// assert_eq!(unsafe { Unchecked::add(i8::MAX - 1, 1i8) }, i8::MAX);
		check_add(i8::MAX - 1, 1, i8::MAX);
		check_add(i16::MAX - 1, 1, i16::MAX);
		check_add(i32::MAX - 1, 1, i32::MAX);
		check_add(i64::MAX - 1, 1, i64::MAX);
		check_add(i128::MAX - 1, 1, i128::MAX);
		check_add(isize::MAX - 1, 1, isize::MAX);
	}
}
