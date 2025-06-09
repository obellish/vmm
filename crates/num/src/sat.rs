use core::{
	fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult, UpperHex},
	ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ops::{
	SaturatingAdd, SaturatingAddAssign, SaturatingDiv, SaturatingDivAssign, SaturatingMul,
	SaturatingMulAssign, SaturatingSub, SaturatingSubAssign,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Saturating<T>(pub T);

impl<T> Saturating<T> {
	pub fn add<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: SaturatingAdd<Rhs>,
	{
		Add::add(Self(lhs), rhs).0
	}

	pub fn sub<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: SaturatingSub<Rhs>,
	{
		Sub::sub(Self(lhs), rhs).0
	}

	pub fn mul<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: SaturatingMul<Rhs>,
	{
		Mul::mul(Self(lhs), rhs).0
	}

	pub fn div<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: SaturatingDiv<Rhs>,
	{
		Div::div(Self(lhs), rhs).0
	}
}

impl<T, Rhs> Add<Rhs> for Saturating<T>
where
	T: SaturatingAdd<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn add(self, rhs: Rhs) -> Self::Output {
		Saturating(SaturatingAdd::saturating_add(self.0, rhs))
	}
}

impl<T, Rhs> AddAssign<Rhs> for Saturating<T>
where
	T: SaturatingAddAssign<Rhs>,
{
	fn add_assign(&mut self, rhs: Rhs) {
		SaturatingAddAssign::saturating_add_assign(&mut self.0, rhs);
	}
}

#[cfg(feature = "arbitrary")]
impl<'a, T> arbitrary::Arbitrary<'a> for Saturating<T>
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

impl<T: Binary> Binary for Saturating<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Binary::fmt(&self.0, f)
	}
}

impl<T: Debug> Debug for Saturating<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'de, T> Deserialize<'de> for Saturating<T>
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

impl<T: Display> Display for Saturating<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<T, Rhs> Div<Rhs> for Saturating<T>
where
	T: SaturatingDiv<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn div(self, rhs: Rhs) -> Self::Output {
		Saturating(SaturatingDiv::saturating_div(self.0, rhs))
	}
}

impl<T, Rhs> DivAssign<Rhs> for Saturating<T>
where
	T: SaturatingDivAssign<Rhs>,
{
	fn div_assign(&mut self, rhs: Rhs) {
		SaturatingDivAssign::saturating_div_assign(&mut self.0, rhs);
	}
}

impl<T> From<T> for Saturating<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<T: LowerHex> LowerHex for Saturating<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		LowerHex::fmt(&self.0, f)
	}
}

impl<T, Rhs> Mul<Rhs> for Saturating<T>
where
	T: SaturatingMul<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn mul(self, rhs: Rhs) -> Self::Output {
		Saturating(SaturatingMul::saturating_mul(self.0, rhs))
	}
}

impl<T, Rhs> MulAssign<Rhs> for Saturating<T>
where
	T: SaturatingMulAssign<Rhs>,
{
	fn mul_assign(&mut self, rhs: Rhs) {
		SaturatingMulAssign::saturating_mul_assign(&mut self.0, rhs);
	}
}

impl<T: Octal> Octal for Saturating<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Octal::fmt(&self.0, f)
	}
}

impl<T: PartialEq> PartialEq<T> for Saturating<T> {
	fn eq(&self, other: &T) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl<T: PartialOrd> PartialOrd<T> for Saturating<T> {
	fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
		PartialOrd::partial_cmp(&self.0, other)
	}
}

impl<T, Rhs> SaturatingAdd<Rhs> for Saturating<T>
where
	T: SaturatingAdd<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn saturating_add(self, rhs: Rhs) -> Self::Output {
		Add::add(self, rhs)
	}
}

impl<T, Rhs> SaturatingAddAssign<Rhs> for Saturating<T>
where
	T: SaturatingAddAssign<Rhs>,
{
	fn saturating_add_assign(&mut self, rhs: Rhs) {
		AddAssign::add_assign(self, rhs);
	}
}

impl<T, Rhs> SaturatingDiv<Rhs> for Saturating<T>
where
	T: SaturatingDiv<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn saturating_div(self, rhs: Rhs) -> Self::Output {
		Div::div(self, rhs)
	}
}

impl<T, Rhs> SaturatingDivAssign<Rhs> for Saturating<T>
where
	T: SaturatingDivAssign<Rhs>,
{
	fn saturating_div_assign(&mut self, rhs: Rhs) {
		DivAssign::div_assign(self, rhs);
	}
}

impl<T, Rhs> SaturatingMul<Rhs> for Saturating<T>
where
	T: SaturatingMul<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn saturating_mul(self, rhs: Rhs) -> Self::Output {
		Mul::mul(self, rhs)
	}
}

impl<T, Rhs> SaturatingMulAssign<Rhs> for Saturating<T>
where
	T: SaturatingMulAssign<Rhs>,
{
	fn saturating_mul_assign(&mut self, rhs: Rhs) {
		MulAssign::mul_assign(self, rhs);
	}
}

impl<T, Rhs> SaturatingSub<Rhs> for Saturating<T>
where
	T: SaturatingSub<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn saturating_sub(self, rhs: Rhs) -> Self::Output {
		Sub::sub(self, rhs)
	}
}

impl<T, Rhs> SaturatingSubAssign<Rhs> for Saturating<T>
where
	T: SaturatingSubAssign<Rhs>,
{
	fn saturating_sub_assign(&mut self, rhs: Rhs) {
		SubAssign::sub_assign(self, rhs);
	}
}

impl<T: Serialize> Serialize for Saturating<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		T::serialize(&self.0, serializer)
	}
}

impl<T, Rhs> Sub<Rhs> for Saturating<T>
where
	T: SaturatingSub<Rhs>,
{
	type Output = Saturating<T::Output>;

	fn sub(self, rhs: Rhs) -> Self::Output {
		Saturating(SaturatingSub::saturating_sub(self.0, rhs))
	}
}

impl<T, Rhs> SubAssign<Rhs> for Saturating<T>
where
	T: SaturatingSubAssign<Rhs>,
{
	fn sub_assign(&mut self, rhs: Rhs) {
		SaturatingSubAssign::saturating_sub_assign(&mut self.0, rhs);
	}
}

impl<T: UpperHex> UpperHex for Saturating<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		UpperHex::fmt(&self.0, f)
	}
}
