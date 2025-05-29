#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(mixed_integer_ops_unsigned_sub))]
#![no_std]

pub mod ops;

use core::{
	cmp::Ordering,
	fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult, UpperHex},
	ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::ops::{
	WrappingAdd, WrappingAddAssign, WrappingDiv, WrappingDivAssign, WrappingMul, WrappingMulAssign,
	WrappingSub, WrappingSubAssign,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Wrapping<T>(pub T);

impl<T> Wrapping<T> {
	pub fn add<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingAdd<Rhs>,
	{
		(Self(lhs) + rhs).0
	}

	pub fn sub<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingSub<Rhs>,
	{
		(Self(lhs) - rhs).0
	}

	pub fn mul<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingMul<Rhs>,
	{
		(Self(lhs) * rhs).0
	}

	pub fn div<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingDiv<Rhs>,
	{
		(Self(lhs) / rhs).0
	}
}

impl<T, Rhs> Add<Rhs> for Wrapping<T>
where
	T: WrappingAdd<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn add(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_add(rhs))
	}
}

impl<T, Rhs> AddAssign<Rhs> for Wrapping<T>
where
	T: WrappingAddAssign<Rhs>,
{
	fn add_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_add_assign(rhs);
	}
}

impl<T: Binary> Binary for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Binary::fmt(&self.0, f)
	}
}

impl<T: Debug> Debug for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'de, T> Deserialize<'de> for Wrapping<T>
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

impl<T: Display> Display for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<T, Rhs> Div<Rhs> for Wrapping<T>
where
	T: WrappingDiv<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn div(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_div(rhs))
	}
}

impl<T, Rhs> DivAssign<Rhs> for Wrapping<T>
where
	T: WrappingDivAssign<Rhs>,
{
	fn div_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_div_assign(rhs);
	}
}

impl<T: LowerHex> LowerHex for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		LowerHex::fmt(&self.0, f)
	}
}

impl<T, Rhs> Mul<Rhs> for Wrapping<T>
where
	T: WrappingMul<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn mul(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_mul(rhs))
	}
}

impl<T, Rhs> MulAssign<Rhs> for Wrapping<T>
where
	T: WrappingMulAssign<Rhs>,
{
	fn mul_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_mul_assign(rhs);
	}
}

impl<T: Octal> Octal for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Octal::fmt(&self.0, f)
	}
}

impl<T: PartialEq> PartialEq<T> for Wrapping<T> {
	fn eq(&self, other: &T) -> bool {
		self.0.eq(other)
	}
}

impl<T: PartialOrd> PartialOrd<T> for Wrapping<T> {
	fn partial_cmp(&self, other: &T) -> Option<Ordering> {
		self.0.partial_cmp(other)
	}
}

impl<T: Serialize> Serialize for Wrapping<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.0.serialize(serializer)
	}
}

impl<T, Rhs> Sub<Rhs> for Wrapping<T>
where
	T: WrappingSub<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn sub(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_sub(rhs))
	}
}

impl<T, Rhs> SubAssign<Rhs> for Wrapping<T>
where
	T: WrappingSubAssign<Rhs>,
{
	fn sub_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_sub_assign(rhs);
	}
}

impl<T: UpperHex> UpperHex for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		UpperHex::fmt(&self.0, f)
	}
}

impl<T, Rhs> WrappingAdd<Rhs> for Wrapping<T>
where
	T: WrappingAdd<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_add(self, rhs: Rhs) -> Self::Output {
		self + rhs
	}
}

impl<T, Rhs> WrappingAddAssign<Rhs> for Wrapping<T>
where
	T: WrappingAddAssign<Rhs>,
{
	fn wrapping_add_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_add_assign(rhs);
	}
}
