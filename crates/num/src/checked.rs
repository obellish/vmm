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
	CheckedAdd, CheckedAddAssign, CheckedDiv, CheckedDivAssign, CheckedMul, CheckedMulAssign,
	CheckedNeg, CheckedRem, CheckedRemAssign, CheckedShl, CheckedShlAssign, CheckedShr,
	CheckedShrAssign, CheckedSub, CheckedSubAssign,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Checked<T>(pub T);

impl<T> Checked<T> {
	pub fn add<Rhs>(lhs: T, rhs: Rhs) -> Option<T::Output>
	where
		T: CheckedAdd<Rhs>,
	{
		Some(Add::add(Self(lhs), rhs)?.0)
	}

	pub fn sub<Rhs>(lhs: T, rhs: Rhs) -> Option<T::Output>
	where
		T: CheckedSub<Rhs>,
	{
		Some(Sub::sub(Self(lhs), rhs)?.0)
	}

	pub fn mul<Rhs>(lhs: T, rhs: Rhs) -> Option<T::Output>
	where
		T: CheckedMul<Rhs>,
	{
		Some(Mul::mul(Self(lhs), rhs)?.0)
	}

	pub fn div<Rhs>(lhs: T, rhs: Rhs) -> Option<T::Output>
	where
		T: CheckedDiv<Rhs>,
	{
		Some(Div::div(Self(lhs), rhs)?.0)
	}

	pub fn neg(lhs: T) -> Option<T::Output>
	where
		T: CheckedNeg,
	{
		Some(Neg::neg(Self(lhs))?.0)
	}

	pub fn shr<Rhs>(lhs: T, rhs: Rhs) -> Option<T::Output>
	where
		T: CheckedShr<Rhs>,
	{
		Some(Shr::shr(Self(lhs), rhs)?.0)
	}

	pub fn shl<Rhs>(lhs: T, rhs: Rhs) -> Option<T::Output>
	where
		T: CheckedShl<Rhs>,
	{
		Some(Shl::shl(Self(lhs), rhs)?.0)
	}
}

impl<T, Rhs> Add<Rhs> for Checked<T>
where
	T: CheckedAdd<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn add(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedAdd::checked_add(self.0, rhs)?))
	}
}

impl<T, Rhs> AddAssign<Rhs> for Checked<T>
where
	T: CheckedAddAssign<Rhs>,
{
	fn add_assign(&mut self, rhs: Rhs) {
		CheckedAddAssign::checked_add_assign(&mut self.0, rhs);
	}
}

#[cfg(feature = "arbitrary")]
impl<'a, T> arbitrary::Arbitrary<'a> for Checked<T>
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

impl<T: Binary> Binary for Checked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Binary::fmt(&self.0, f)
	}
}

impl<T, Rhs> CheckedAdd<Rhs> for Checked<T>
where
	T: CheckedAdd<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_add(self, rhs: Rhs) -> Option<Self::Output> {
		Add::add(self, rhs)
	}
}

impl<T, Rhs> CheckedAddAssign<Rhs> for Checked<T>
where
	T: CheckedAddAssign<Rhs>,
{
	fn checked_add_assign(&mut self, rhs: Rhs) {
		AddAssign::add_assign(self, rhs);
	}
}

impl<T, Rhs> CheckedDiv<Rhs> for Checked<T>
where
	T: CheckedDiv<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_div(self, rhs: Rhs) -> Option<Self::Output> {
		Div::div(self, rhs)
	}
}

impl<T, Rhs> CheckedDivAssign<Rhs> for Checked<T>
where
	T: CheckedDivAssign<Rhs>,
{
	fn checked_div_assign(&mut self, rhs: Rhs) {
		DivAssign::div_assign(self, rhs);
	}
}

impl<T, Rhs> CheckedMul<Rhs> for Checked<T>
where
	T: CheckedMul<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_mul(self, rhs: Rhs) -> Option<Self::Output> {
		Mul::mul(self, rhs)
	}
}

impl<T, Rhs> CheckedMulAssign<Rhs> for Checked<T>
where
	T: CheckedMulAssign<Rhs>,
{
	fn checked_mul_assign(&mut self, rhs: Rhs) {
		MulAssign::mul_assign(self, rhs);
	}
}

impl<T: CheckedNeg> CheckedNeg for Checked<T> {
	type Output = Checked<T::Output>;

	fn checked_neg(self) -> Option<Self::Output> {
		Neg::neg(self)
	}
}

impl<T, Rhs> CheckedRem<Rhs> for Checked<T>
where
	T: CheckedRem<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_rem(self, rhs: Rhs) -> Option<Self::Output> {
		Rem::rem(self, rhs)
	}
}

impl<T, Rhs> CheckedRemAssign<Rhs> for Checked<T>
where
	T: CheckedRemAssign<Rhs>,
{
	fn checked_rem_assign(&mut self, rhs: Rhs) {
		RemAssign::rem_assign(self, rhs);
	}
}

impl<T, Rhs> CheckedShl<Rhs> for Checked<T>
where
	T: CheckedShl<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_shl(self, rhs: Rhs) -> Option<Self::Output> {
		Shl::shl(self, rhs)
	}
}

impl<T, Rhs> CheckedShlAssign<Rhs> for Checked<T>
where
	T: CheckedShlAssign<Rhs>,
{
	fn checked_shl_assign(&mut self, rhs: Rhs) {
		ShlAssign::shl_assign(self, rhs);
	}
}

impl<T, Rhs> CheckedShr<Rhs> for Checked<T>
where
	T: CheckedShr<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_shr(self, rhs: Rhs) -> Option<Self::Output> {
		Shr::shr(self, rhs)
	}
}

impl<T, Rhs> CheckedShrAssign<Rhs> for Checked<T>
where
	T: CheckedShrAssign<Rhs>,
{
	fn checked_shr_assign(&mut self, rhs: Rhs) {
		ShrAssign::shr_assign(self, rhs);
	}
}

impl<T, Rhs> CheckedSub<Rhs> for Checked<T>
where
	T: CheckedSub<Rhs>,
{
	type Output = Checked<T::Output>;

	fn checked_sub(self, rhs: Rhs) -> Option<Self::Output> {
		Sub::sub(self, rhs)
	}
}

impl<T, Rhs> CheckedSubAssign<Rhs> for Checked<T>
where
	T: CheckedSubAssign<Rhs>,
{
	fn checked_sub_assign(&mut self, rhs: Rhs) {
		SubAssign::sub_assign(self, rhs);
	}
}

impl<T: Debug> Debug for Checked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'de, T> Deserialize<'de> for Checked<T>
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

impl<T: Display> Display for Checked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<T, Rhs> Div<Rhs> for Checked<T>
where
	T: CheckedDiv<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn div(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedDiv::checked_div(self.0, rhs)?))
	}
}

impl<T, Rhs> DivAssign<Rhs> for Checked<T>
where
	T: CheckedDivAssign<Rhs>,
{
	fn div_assign(&mut self, rhs: Rhs) {
		CheckedDivAssign::checked_div_assign(&mut self.0, rhs);
	}
}

impl<T> From<T> for Checked<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<T: LowerHex> LowerHex for Checked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		LowerHex::fmt(&self.0, f)
	}
}

impl<T, Rhs> Mul<Rhs> for Checked<T>
where
	T: CheckedMul<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn mul(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedMul::checked_mul(self.0, rhs)?))
	}
}

impl<T, Rhs> MulAssign<Rhs> for Checked<T>
where
	T: CheckedMulAssign<Rhs>,
{
	fn mul_assign(&mut self, rhs: Rhs) {
		CheckedMulAssign::checked_mul_assign(&mut self.0, rhs);
	}
}

impl<T: CheckedNeg> Neg for Checked<T> {
	type Output = Option<Checked<T::Output>>;

	fn neg(self) -> Self::Output {
		Some(Checked(CheckedNeg::checked_neg(self.0)?))
	}
}

impl<T: Octal> Octal for Checked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Octal::fmt(&self.0, f)
	}
}

impl<T: PartialEq> PartialEq<T> for Checked<T> {
	fn eq(&self, other: &T) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl<T: PartialOrd> PartialOrd<T> for Checked<T> {
	fn partial_cmp(&self, other: &T) -> Option<Ordering> {
		PartialOrd::partial_cmp(&self.0, other)
	}
}

impl<T, Rhs> Rem<Rhs> for Checked<T>
where
	T: CheckedRem<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn rem(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedRem::checked_rem(self.0, rhs)?))
	}
}

impl<T, Rhs> RemAssign<Rhs> for Checked<T>
where
	T: CheckedRemAssign<Rhs>,
{
	fn rem_assign(&mut self, rhs: Rhs) {
		CheckedRemAssign::checked_rem_assign(&mut self.0, rhs);
	}
}

impl<T: Serialize> Serialize for Checked<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Serialize::serialize(&self.0, serializer)
	}
}

impl<T, Rhs> Shl<Rhs> for Checked<T>
where
	T: CheckedShl<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn shl(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedShl::checked_shl(self.0, rhs)?))
	}
}

impl<T, Rhs> ShlAssign<Rhs> for Checked<T>
where
	T: CheckedShlAssign<Rhs>,
{
	fn shl_assign(&mut self, rhs: Rhs) {
		CheckedShlAssign::checked_shl_assign(&mut self.0, rhs);
	}
}

impl<T, Rhs> Shr<Rhs> for Checked<T>
where
	T: CheckedShr<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn shr(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedShr::checked_shr(self.0, rhs)?))
	}
}

impl<T, Rhs> ShrAssign<Rhs> for Checked<T>
where
	T: CheckedShrAssign<Rhs>,
{
	fn shr_assign(&mut self, rhs: Rhs) {
		CheckedShrAssign::checked_shr_assign(&mut self.0, rhs);
	}
}

impl<T, Rhs> Sub<Rhs> for Checked<T>
where
	T: CheckedSub<Rhs>,
{
	type Output = Option<Checked<T::Output>>;

	fn sub(self, rhs: Rhs) -> Self::Output {
		Some(Checked(CheckedSub::checked_sub(self.0, rhs)?))
	}
}

impl<T, Rhs> SubAssign<Rhs> for Checked<T>
where
	T: CheckedSubAssign<Rhs>,
{
	fn sub_assign(&mut self, rhs: Rhs) {
		CheckedSubAssign::checked_sub_assign(&mut self.0, rhs);
	}
}

impl<T: UpperHex> UpperHex for Checked<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		UpperHex::fmt(&self.0, f)
	}
}
