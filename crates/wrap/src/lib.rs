#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(mixed_integer_ops_unsigned_sub))]
#![no_std]

pub mod ops;

use core::{
	cmp::Ordering,
	fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult, UpperHex},
	ops::{
		Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign,
		Shr, ShrAssign, Sub, SubAssign,
	},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::ops::{
	WrappingAdd, WrappingAddAssign, WrappingDiv, WrappingDivAssign, WrappingMul, WrappingMulAssign,
	WrappingRem, WrappingRemAssign, WrappingShl, WrappingShlAssign, WrappingShr, WrappingShrAssign,
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

	pub fn rem<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingRem<Rhs>,
	{
		(Self(lhs) % rhs).0
	}

	pub fn shl<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingShl<Rhs>,
	{
		(Self(lhs) << rhs).0
	}

	pub fn shr<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: WrappingShr<Rhs>,
	{
		(Self(lhs) >> rhs).0
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

impl<T: Neg> Neg for Wrapping<T> {
	type Output = Wrapping<T::Output>;

	fn neg(self) -> Self::Output {
		Wrapping(self.0.neg())
	}
}

impl<T: Not> Not for Wrapping<T> {
	type Output = Wrapping<T::Output>;

	fn not(self) -> Self::Output {
		Wrapping(self.0.not())
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

impl<T, Rhs> Rem<Rhs> for Wrapping<T>
where
	T: WrappingRem<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn rem(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_rem(rhs))
	}
}

impl<T, Rhs> RemAssign<Rhs> for Wrapping<T>
where
	T: WrappingRemAssign<Rhs>,
{
	fn rem_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_rem_assign(rhs);
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

impl<T, Rhs> Shl<Rhs> for Wrapping<T>
where
	T: WrappingShl<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn shl(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_shl(rhs))
	}
}

impl<T, Rhs> ShlAssign<Rhs> for Wrapping<T>
where
	T: WrappingShlAssign<Rhs>,
{
	fn shl_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_shl_assign(rhs);
	}
}

impl<T, Rhs> Shr<Rhs> for Wrapping<T>
where
	T: WrappingShr<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn shr(self, rhs: Rhs) -> Self::Output {
		Wrapping(self.0.wrapping_shr(rhs))
	}
}

impl<T, Rhs> ShrAssign<Rhs> for Wrapping<T>
where
	T: WrappingShrAssign<Rhs>,
{
	fn shr_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_shr_assign(rhs);
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

impl<T, Rhs> WrappingDiv<Rhs> for Wrapping<T>
where
	T: WrappingDiv<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_div(self, rhs: Rhs) -> Self::Output {
		self / rhs
	}
}

impl<T, Rhs> WrappingDivAssign<Rhs> for Wrapping<T>
where
	T: WrappingDivAssign<Rhs>,
{
	fn wrapping_div_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_div_assign(rhs);
	}
}

impl<T, Rhs> WrappingMul<Rhs> for Wrapping<T>
where
	T: WrappingMul<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_mul(self, rhs: Rhs) -> Self::Output {
		self * rhs
	}
}

impl<T, Rhs> WrappingMulAssign<Rhs> for Wrapping<T>
where
	T: WrappingMulAssign<Rhs>,
{
	fn wrapping_mul_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_mul_assign(rhs);
	}
}

impl<T, Rhs> WrappingRem<Rhs> for Wrapping<T>
where
	T: WrappingRem<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_rem(self, rhs: Rhs) -> Self::Output {
		self % rhs
	}
}

impl<T, Rhs> WrappingRemAssign<Rhs> for Wrapping<T>
where
	T: WrappingRemAssign<Rhs>,
{
	fn wrapping_rem_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_rem_assign(rhs);
	}
}

impl<T, Rhs> WrappingShl<Rhs> for Wrapping<T>
where
	T: WrappingShl<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_shl(self, rhs: Rhs) -> Self::Output {
		self << rhs
	}
}

impl<T, Rhs> WrappingShlAssign<Rhs> for Wrapping<T>
where
	T: WrappingShlAssign<Rhs>,
{
	fn wrapping_shl_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_shl_assign(rhs);
	}
}

impl<T, Rhs> WrappingShr<Rhs> for Wrapping<T>
where
	T: WrappingShr<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_shr(self, rhs: Rhs) -> Self::Output {
		self >> rhs
	}
}

impl<T, Rhs> WrappingShrAssign<Rhs> for Wrapping<T>
where
	T: WrappingShrAssign<Rhs>,
{
	fn wrapping_shr_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_shr_assign(rhs);
	}
}

impl<T, Rhs> WrappingSub<Rhs> for Wrapping<T>
where
	T: WrappingSub<Rhs>,
{
	type Output = Wrapping<T::Output>;

	fn wrapping_sub(self, rhs: Rhs) -> Self::Output {
		self - rhs
	}
}

impl<T, Rhs> WrappingSubAssign<Rhs> for Wrapping<T>
where
	T: WrappingSubAssign<Rhs>,
{
	fn wrapping_sub_assign(&mut self, rhs: Rhs) {
		self.0.wrapping_sub_assign(rhs);
	}
}
