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
	CastTo, WrappingAdd, WrappingAddAssign, WrappingDiv, WrappingDivAssign, WrappingMul,
	WrappingMulAssign, WrappingNeg, WrappingRem, WrappingRemAssign, WrappingShl, WrappingShlAssign,
	WrappingShr, WrappingShrAssign, WrappingSub, WrappingSubAssign,
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

impl<T, To> CastTo<Wrapping<To>> for Wrapping<T>
where
	T: CastTo<To>,
{
	fn cast(self) -> Wrapping<To> {
		Wrapping(CastTo::cast(self.0))
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

#[cfg(test)]
mod tests {
	use core::fmt::Debug;

	use crate::{
		Wrapping,
		ops::{WrappingAdd, WrappingMul, WrappingSub},
	};

	fn check_add<T, Rhs>(value: T, one: Rhs, expected: T)
	where
		T: Debug + PartialEq + WrappingAdd<Rhs, Output = T>,
	{
		assert_eq!(Wrapping::add(value, one), expected);
	}

	fn check_sub<T, Rhs>(value: T, one: Rhs, expected: T)
	where
		T: Debug + PartialEq + WrappingSub<Rhs, Output = T>,
	{
		assert_eq!(Wrapping::sub(value, one), expected);
	}

	fn check_mul<T, Rhs>(value: T, multiplier: Rhs, expected: T)
	where
		T: Debug + PartialEq + WrappingMul<Rhs, Output = T>,
	{
		assert_eq!(Wrapping::mul(value, multiplier), expected);
	}

	#[test]
	fn add_signed() {
		check_add(i8::MAX, 1i8, i8::MIN);
		check_add(i16::MAX, 1i16, i16::MIN);
		check_add(i32::MAX, 1i32, i32::MIN);
		check_add(i64::MAX, 1i64, i64::MIN);
		check_add(i128::MAX, 1i128, i128::MIN);
		check_add(isize::MAX, 1isize, isize::MIN);
	}

	#[test]
	fn add_signed_unsigned() {
		check_add(i8::MAX, u8::MAX, 126);
		check_add(i16::MAX, u16::MAX, 32766);
		check_add(i32::MAX, u32::MAX, 2_147_483_646);
		check_add(i64::MAX, u64::MAX, 9_223_372_036_854_775_806);
		check_add(
			i128::MAX,
			u128::MAX,
			170_141_183_460_469_231_731_687_303_715_884_105_726,
		);
		check_add(isize::MAX, usize::MAX, 9_223_372_036_854_775_806);
	}

	#[test]
	fn add_unsigned() {
		check_add(u8::MAX, 1u8, u8::MIN);
		check_add(u16::MAX, 1u16, u16::MIN);
		check_add(u32::MAX, 1u32, u32::MIN);
		check_add(u64::MAX, 1u64, u64::MIN);
		check_add(u128::MAX, 1u128, u128::MIN);
		check_add(usize::MAX, 1usize, usize::MIN);
	}

	#[test]
	fn add_unsigned_signed() {
		check_add(u8::MAX, i8::MAX, 126);
		check_add(u16::MAX, i16::MAX, 32766);
		check_add(u32::MAX, i32::MAX, 2_147_483_646);
		check_add(u64::MAX, i64::MAX, 9_223_372_036_854_775_806);
		check_add(
			u128::MAX,
			i128::MAX,
			170_141_183_460_469_231_731_687_303_715_884_105_726,
		);
		check_add(usize::MAX, isize::MAX, 9_223_372_036_854_775_806);
	}

	#[test]
	fn sub_signed() {
		check_sub(i8::MIN, 1i8, i8::MAX);
		check_sub(i16::MIN, 1i16, i16::MAX);
		check_sub(i32::MIN, 1i32, i32::MAX);
		check_sub(i64::MIN, 1i64, i64::MAX);
		check_sub(i128::MIN, 1i128, i128::MAX);
		check_sub(isize::MIN, 1isize, isize::MAX);
	}

	#[test]
	fn sub_signed_unsigned() {
		check_sub(i8::MAX, u8::MAX, i8::MIN);
		check_sub(i16::MAX, u16::MAX, i16::MIN);
		check_sub(i32::MAX, u32::MAX, i32::MIN);
		check_sub(i64::MAX, u64::MAX, i64::MIN);
		check_sub(i128::MAX, u128::MAX, i128::MIN);
		check_sub(isize::MAX, usize::MAX, isize::MIN);
	}

	#[test]
	fn sub_unsigned() {
		check_sub(u8::MIN, 1u8, u8::MAX);
		check_sub(u16::MIN, 1u16, u16::MAX);
		check_sub(u32::MIN, 1u32, u32::MAX);
		check_sub(u64::MIN, 1u64, u64::MAX);
		check_sub(u128::MIN, 1u128, u128::MAX);
		check_sub(usize::MIN, 1usize, usize::MAX);
	}

	#[test]
	#[cfg(feature = "nightly")]
	fn sub_unsigned_signed() {
		check_sub(u8::MAX, i8::MAX, 128);
		check_sub(u16::MAX, i16::MAX, 32768);
		check_sub(u32::MAX, i32::MAX, 2_147_483_648);
		check_sub(u64::MAX, i64::MAX, 9_223_372_036_854_775_808);
		check_sub(
			u128::MAX,
			i128::MAX,
			170_141_183_460_469_231_731_687_303_715_884_105_728,
		);
		check_sub(usize::MAX, isize::MAX, 9_223_372_036_854_775_808);
	}

	#[test]
	fn mul_signed() {
		check_mul(0xfeu8 as i8, 16, 0xe0u8 as i8);
		check_mul(0xfedcu16 as i16, 16, 0xedc0u16 as i16);
		check_mul(0xfedc_ba98u32 as i32, 16, 0xedcb_a980_u32 as i32);
		check_mul(
			0xfedc_ba98_7654_3217u64 as i64,
			16,
			0xedcb_a987_6543_2170u64 as i64,
		);
	}

	#[test]
	fn mul_unsigned() {
		check_mul(0xfeu8, 16, 0xe0);
		check_mul(0xfedcu16, 16, 0xedc0);
		check_mul(0xfedc_ba98_u32, 16, 0xedcb_a980);
		check_mul(0xfedc_ba98_7654_3217u64, 16, 0xedcb_a987_6543_2170);
	}
}
