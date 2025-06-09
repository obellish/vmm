use core::{
	cmp::Ordering,
	fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult, UpperHex},
	ops::{
		Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr,
		ShrAssign, Sub, SubAssign,
	},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ops::{
	WrappingAdd, WrappingAddAssign, WrappingDiv, WrappingDivAssign, WrappingMul, WrappingMulAssign,
	WrappingNeg, WrappingRem, WrappingRemAssign, WrappingShl, WrappingShlAssign, WrappingShr,
	WrappingShrAssign, WrappingSub, WrappingSubAssign,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Wrapping<T>(pub T);

impl<T> Wrapping<T> {
	pub fn add<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingAdd<Rhs>,
	{
		Add::add(Self(lhs), rhs).0
	}

	pub fn sub<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingSub<Rhs>,
	{
		Sub::sub(Self(lhs), rhs).0
	}

	pub fn mul<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingMul<Rhs>,
	{
		Mul::mul(Self(lhs), rhs).0
	}

	pub fn div<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingDiv<Rhs>,
	{
		Div::div(Self(lhs), rhs).0
	}

	pub fn neg(lhs: T) -> T::Output
	where
		T: WrappingNeg,
	{
		Neg::neg(Self(lhs)).0
	}

	pub fn shr<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingShr<Rhs>,
	{
		Shr::shr(Self(lhs), rhs).0
	}

	pub fn shl<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingShl<Rhs>,
	{
		Shl::shl(Self(lhs), rhs).0
	}
}

impl<T, Rhs> Add<Rhs> for Wrapping<T>
where
	T: WrappingAdd<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn add(self, rhs: Rhs) -> Self::Output {
		Wrapping(WrappingAdd::wrapping_add(self.0, rhs))
	}
}

impl<T, Rhs> AddAssign<Rhs> for Wrapping<T>
where
	T: WrappingAddAssign<Rhs>,
{
	fn add_assign(&mut self, rhs: Rhs) {
		WrappingAddAssign::wrapping_add_assign(&mut self.0, rhs);
	}
}

#[cfg(feature = "arbitrary")]
impl<'a, T> arbitrary::Arbitrary<'a> for Wrapping<T>
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
		Wrapping(WrappingDiv::wrapping_div(self.0, rhs))
	}
}

impl<T, Rhs> DivAssign<Rhs> for Wrapping<T>
where
	T: WrappingDivAssign<Rhs>,
{
	fn div_assign(&mut self, rhs: Rhs) {
		WrappingDivAssign::wrapping_div_assign(&mut self.0, rhs);
	}
}

impl<T> From<T> for Wrapping<T> {
	fn from(value: T) -> Self {
		Self(value)
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
		Wrapping(WrappingMul::wrapping_mul(self.0, rhs))
	}
}

impl<T, Rhs> MulAssign<Rhs> for Wrapping<T>
where
	T: WrappingMulAssign<Rhs>,
{
	fn mul_assign(&mut self, rhs: Rhs) {
		WrappingMulAssign::wrapping_mul_assign(&mut self.0, rhs);
	}
}

impl<T: WrappingNeg> Neg for Wrapping<T> {
	type Output = Wrapping<T::Output>;

	fn neg(self) -> Self::Output {
		Wrapping(WrappingNeg::wrapping_neg(self.0))
	}
}

impl<T: Octal> Octal for Wrapping<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Octal::fmt(&self.0, f)
	}
}

impl<T: PartialEq> PartialEq<T> for Wrapping<T> {
	fn eq(&self, other: &T) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl<T: PartialOrd> PartialOrd<T> for Wrapping<T> {
	fn partial_cmp(&self, other: &T) -> Option<Ordering> {
		PartialOrd::partial_cmp(&self.0, other)
	}
}

impl<T, Rhs> Rem<Rhs> for Wrapping<T>
where
	T: WrappingRem<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn rem(self, rhs: Rhs) -> Self::Output {
		Wrapping(WrappingRem::wrapping_rem(self.0, rhs))
	}
}

impl<T, Rhs> RemAssign<Rhs> for Wrapping<T>
where
	T: WrappingRemAssign<Rhs>,
{
	fn rem_assign(&mut self, rhs: Rhs) {
		WrappingRemAssign::wrapping_rem_assign(&mut self.0, rhs);
	}
}

impl<T: Serialize> Serialize for Wrapping<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Serialize::serialize(&self.0, serializer)
	}
}

impl<T, Rhs> Shl<Rhs> for Wrapping<T>
where
	T: WrappingShl<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn shl(self, rhs: Rhs) -> Self::Output {
		Wrapping(WrappingShl::wrapping_shl(self.0, rhs))
	}
}

impl<T, Rhs> ShlAssign<Rhs> for Wrapping<T>
where
	T: WrappingShlAssign<Rhs>,
{
	fn shl_assign(&mut self, rhs: Rhs) {
		WrappingShlAssign::wrapping_shl_assign(&mut self.0, rhs);
	}
}

impl<T, Rhs> Shr<Rhs> for Wrapping<T>
where
	T: WrappingShr<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn shr(self, rhs: Rhs) -> Self::Output {
		Wrapping(WrappingShr::wrapping_shr(self.0, rhs))
	}
}

impl<T, Rhs> ShrAssign<Rhs> for Wrapping<T>
where
	T: WrappingShrAssign<Rhs>,
{
	fn shr_assign(&mut self, rhs: Rhs) {
		WrappingShrAssign::wrapping_shr_assign(&mut self.0, rhs);
	}
}

impl<T, Rhs> Sub<Rhs> for Wrapping<T>
where
	T: WrappingSub<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn sub(self, rhs: Rhs) -> Self::Output {
		Wrapping(WrappingSub::wrapping_sub(self.0, rhs))
	}
}

impl<T, Rhs> SubAssign<Rhs> for Wrapping<T>
where
	T: WrappingSubAssign<Rhs>,
{
	fn sub_assign(&mut self, rhs: Rhs) {
		WrappingSubAssign::wrapping_sub_assign(&mut self.0, rhs);
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
		Add::add(self, rhs)
	}
}

impl<T, Rhs> WrappingAddAssign<Rhs> for Wrapping<T>
where
	T: WrappingAddAssign<Rhs>,
{
	fn wrapping_add_assign(&mut self, rhs: Rhs) {
		AddAssign::add_assign(self, rhs);
	}
}

impl<T, Rhs> WrappingDiv<Rhs> for Wrapping<T>
where
	T: WrappingDiv<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_div(self, rhs: Rhs) -> Self::Output {
		Div::div(self, rhs)
	}
}

impl<T, Rhs> WrappingDivAssign<Rhs> for Wrapping<T>
where
	T: WrappingDivAssign<Rhs>,
{
	fn wrapping_div_assign(&mut self, rhs: Rhs) {
		DivAssign::div_assign(self, rhs);
	}
}

impl<T, Rhs> WrappingMul<Rhs> for Wrapping<T>
where
	T: WrappingMul<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_mul(self, rhs: Rhs) -> Self::Output {
		Mul::mul(self, rhs)
	}
}

impl<T, Rhs> WrappingMulAssign<Rhs> for Wrapping<T>
where
	T: WrappingMulAssign<Rhs>,
{
	fn wrapping_mul_assign(&mut self, rhs: Rhs) {
		MulAssign::mul_assign(self, rhs);
	}
}

impl<T: WrappingNeg> WrappingNeg for Wrapping<T> {
	type Output = Wrapping<T::Output>;

	fn wrapping_neg(self) -> Self::Output {
		Neg::neg(self)
	}
}

impl<T, Rhs> WrappingRem<Rhs> for Wrapping<T>
where
	T: WrappingRem<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_rem(self, rhs: Rhs) -> Self::Output {
		Rem::rem(self, rhs)
	}
}

impl<T, Rhs> WrappingRemAssign<Rhs> for Wrapping<T>
where
	T: WrappingRemAssign<Rhs>,
{
	fn wrapping_rem_assign(&mut self, rhs: Rhs) {
		RemAssign::rem_assign(self, rhs);
	}
}

impl<T, Rhs> WrappingShl<Rhs> for Wrapping<T>
where
	T: WrappingShl<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_shl(self, rhs: Rhs) -> Self::Output {
		Shl::shl(self, rhs)
	}
}

impl<T, Rhs> WrappingShlAssign<Rhs> for Wrapping<T>
where
	T: WrappingShlAssign<Rhs>,
{
	fn wrapping_shl_assign(&mut self, rhs: Rhs) {
		ShlAssign::shl_assign(self, rhs);
	}
}

impl<T, Rhs> WrappingShr<Rhs> for Wrapping<T>
where
	T: WrappingShr<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_shr(self, rhs: Rhs) -> Self::Output {
		Shr::shr(self, rhs)
	}
}

impl<T, Rhs> WrappingShrAssign<Rhs> for Wrapping<T>
where
	T: WrappingShrAssign<Rhs>,
{
	fn wrapping_shr_assign(&mut self, rhs: Rhs) {
		ShrAssign::shr_assign(self, rhs);
	}
}

impl<T, Rhs> WrappingSub<Rhs> for Wrapping<T>
where
	T: WrappingSub<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_sub(self, rhs: Rhs) -> Self::Output {
		Sub::sub(self, rhs)
	}
}

impl<T, Rhs> WrappingSubAssign<Rhs> for Wrapping<T>
where
	T: WrappingSubAssign<Rhs>,
{
	fn wrapping_sub_assign(&mut self, rhs: Rhs) {
		SubAssign::sub_assign(self, rhs);
	}
}
