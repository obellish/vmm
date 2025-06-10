use core::{
	cmp::Ordering,
	fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult, UpperHex},
	ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ops::{
	StrictAdd, StrictAddAssign, StrictDiv, StrictDivAssign, StrictMul, StrictMulAssign, StrictNeg,
	StrictRem, StrictRemAssign, StrictSub, StrictSubAssign,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Strict<T>(pub T);

impl<T> Strict<T> {
	pub fn add<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: StrictAdd<Rhs>,
	{
		Add::add(Self(lhs), rhs).0
	}

	pub fn sub<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: StrictSub<Rhs>,
	{
		Sub::sub(Self(lhs), rhs).0
	}

	pub fn mul<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: StrictMul<Rhs>,
	{
		Mul::mul(Self(lhs), rhs).0
	}

	pub fn div<Rhs>(lhs: T, rhs: Rhs) -> T::Output
	where
		T: StrictDiv<Rhs>,
	{
		Div::div(Self(lhs), rhs).0
	}

	pub fn neg(value: T) -> T::Output
	where
		T: StrictNeg,
	{
		Neg::neg(Self(value)).0
	}
}

impl<T, Rhs> Add<Rhs> for Strict<T>
where
	T: StrictAdd<Rhs>,
{
	type Output = Strict<T::Output>;

	fn add(self, rhs: Rhs) -> Self::Output {
		Strict(StrictAdd::strict_add(self.0, rhs))
	}
}

impl<T, Rhs> AddAssign<Rhs> for Strict<T>
where
	T: StrictAddAssign<Rhs>,
{
	fn add_assign(&mut self, rhs: Rhs) {
		StrictAddAssign::strict_add_assign(&mut self.0, rhs);
	}
}

#[cfg(feature = "arbitrary")]
impl<'a, T> arbitrary::Arbitrary<'a> for Strict<T>
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

impl<T: Binary> Binary for Strict<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Binary::fmt(&self.0, f)
	}
}

impl<T: Debug> Debug for Strict<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'de, T> Deserialize<'de> for Strict<T>
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

impl<T: Display> Display for Strict<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<T, Rhs> Div<Rhs> for Strict<T>
where
	T: StrictDiv<Rhs>,
{
	type Output = Strict<T::Output>;

	fn div(self, rhs: Rhs) -> Self::Output {
		Strict(StrictDiv::strict_div(self.0, rhs))
	}
}

impl<T, Rhs> DivAssign<Rhs> for Strict<T>
where
	T: StrictDivAssign<Rhs>,
{
	fn div_assign(&mut self, rhs: Rhs) {
		StrictDivAssign::strict_div_assign(&mut self.0, rhs);
	}
}

impl<T> From<T> for Strict<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<T: LowerHex> LowerHex for Strict<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		LowerHex::fmt(&self.0, f)
	}
}

impl<T, Rhs> Mul<Rhs> for Strict<T>
where
	T: StrictMul<Rhs>,
{
	type Output = Strict<T::Output>;

	fn mul(self, rhs: Rhs) -> Self::Output {
		Strict(StrictMul::strict_mul(self.0, rhs))
	}
}

impl<T, Rhs> MulAssign<Rhs> for Strict<T>
where
	T: StrictMulAssign<Rhs>,
{
	fn mul_assign(&mut self, rhs: Rhs) {
		StrictMulAssign::strict_mul_assign(&mut self.0, rhs);
	}
}

impl<T: StrictNeg> Neg for Strict<T> {
	type Output = Strict<T::Output>;

	fn neg(self) -> Self::Output {
		Strict(StrictNeg::strict_neg(self.0))
	}
}

impl<T: Octal> Octal for Strict<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Octal::fmt(&self.0, f)
	}
}

impl<T: PartialEq> PartialEq<T> for Strict<T> {
	fn eq(&self, other: &T) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl<T: PartialOrd> PartialOrd<T> for Strict<T> {
	fn partial_cmp(&self, other: &T) -> Option<Ordering> {
		PartialOrd::partial_cmp(&self.0, other)
	}
}

impl<T, Rhs> Rem<Rhs> for Strict<T>
where
	T: StrictRem<Rhs>,
{
	type Output = Strict<T::Output>;

	fn rem(self, rhs: Rhs) -> Self::Output {
		Strict(StrictRem::strict_rem(self.0, rhs))
	}
}

impl<T, Rhs> RemAssign<Rhs> for Strict<T>
where
	T: StrictRemAssign<Rhs>,
{
	fn rem_assign(&mut self, rhs: Rhs) {
		StrictRemAssign::strict_rem_assign(&mut self.0, rhs);
	}
}

impl<T: Serialize> Serialize for Strict<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Serialize::serialize(&self.0, serializer)
	}
}

impl<T, Rhs> StrictAdd<Rhs> for Strict<T>
where
	T: StrictAdd<Rhs>,
{
	type Output = Strict<T::Output>;

	fn strict_add(self, rhs: Rhs) -> Self::Output {
		Add::add(self, rhs)
	}
}

impl<T, Rhs> StrictAddAssign<Rhs> for Strict<T>
where
	T: StrictAddAssign<Rhs>,
{
	fn strict_add_assign(&mut self, rhs: Rhs) {
		AddAssign::add_assign(self, rhs);
	}
}

impl<T, Rhs> StrictDiv<Rhs> for Strict<T>
where
	T: StrictDiv<Rhs>,
{
	type Output = Strict<T::Output>;

	fn strict_div(self, rhs: Rhs) -> Self::Output {
		Div::div(self, rhs)
	}
}

impl<T, Rhs> StrictDivAssign<Rhs> for Strict<T>
where
	T: StrictDivAssign<Rhs>,
{
	fn strict_div_assign(&mut self, rhs: Rhs) {
		DivAssign::div_assign(self, rhs);
	}
}

impl<T, Rhs> StrictMul<Rhs> for Strict<T>
where
	T: StrictMul<Rhs>,
{
	type Output = Strict<T::Output>;

	fn strict_mul(self, rhs: Rhs) -> Self::Output {
		Mul::mul(self, rhs)
	}
}

impl<T, Rhs> StrictMulAssign<Rhs> for Strict<T>
where
	T: StrictMulAssign<Rhs>,
{
	fn strict_mul_assign(&mut self, rhs: Rhs) {
		MulAssign::mul_assign(self, rhs);
	}
}

impl<T: StrictNeg> StrictNeg for Strict<T> {
	type Output = Strict<T::Output>;

	fn strict_neg(self) -> Self::Output {
		Neg::neg(self)
	}
}

impl<T, Rhs> StrictRem<Rhs> for Strict<T>
where
	T: StrictRem<Rhs>,
{
	type Output = Strict<T::Output>;

	fn strict_rem(self, rhs: Rhs) -> Self::Output {
		Rem::rem(self, rhs)
	}
}

impl<T, Rhs> StrictRemAssign<Rhs> for Strict<T>
where
	T: StrictRemAssign<Rhs>,
{
	fn strict_rem_assign(&mut self, rhs: Rhs) {
		RemAssign::rem_assign(self, rhs);
	}
}

impl<T, Rhs> StrictSub<Rhs> for Strict<T>
where
	T: StrictSub<Rhs>,
{
	type Output = Strict<T::Output>;

	fn strict_sub(self, rhs: Rhs) -> Self::Output {
		Sub::sub(self, rhs)
	}
}

impl<T, Rhs> StrictSubAssign<Rhs> for Strict<T>
where
	T: StrictSubAssign<Rhs>,
{
	fn strict_sub_assign(&mut self, rhs: Rhs) {
		SubAssign::sub_assign(self, rhs);
	}
}

impl<T, Rhs> Sub<Rhs> for Strict<T>
where
	T: StrictSub<Rhs>,
{
	type Output = Strict<T::Output>;

	fn sub(self, rhs: Rhs) -> Self::Output {
		Strict(StrictSub::strict_sub(self.0, rhs))
	}
}

impl<T, Rhs> SubAssign<Rhs> for Strict<T>
where
	T: StrictSubAssign<Rhs>,
{
	fn sub_assign(&mut self, rhs: Rhs) {
		StrictSubAssign::strict_sub_assign(&mut self.0, rhs);
	}
}

impl<T: UpperHex> UpperHex for Strict<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		UpperHex::fmt(&self.0, f)
	}
}
