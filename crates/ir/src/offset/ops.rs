use core::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign,
};

use vmm_num::ops::{
	CheckedAdd, CheckedAddAssign, CheckedDiv, CheckedDivAssign, CheckedMul, CheckedMulAssign,
	CheckedNeg, CheckedRem, CheckedRemAssign, CheckedSub, CheckedSubAssign, SaturatingAdd,
	SaturatingAddAssign, SaturatingDiv, SaturatingDivAssign, SaturatingMul, WrappingAdd,
};

use super::Offset;

impl Add for Offset {
	type Output = Self;

	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Self(Add::add(self.0, rhs.0))
	}
}

impl Add<&Self> for Offset {
	type Output = Self;

	#[inline]
	fn add(self, rhs: &Self) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<Offset> for &Offset {
	type Output = Offset;

	#[inline]
	fn add(self, rhs: Offset) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add for &Offset {
	type Output = Offset;

	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl Add<isize> for Offset {
	type Output = Self;

	#[inline]
	fn add(self, rhs: isize) -> Self::Output {
		Self(Add::add(self.0, rhs))
	}
}

impl Add<&isize> for Offset {
	type Output = Self;

	#[inline]
	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn add(self, rhs: isize) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add<&isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl AddAssign for Offset {
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		AddAssign::add_assign(&mut self.0, rhs.0);
	}
}

impl AddAssign<&Self> for Offset {
	#[inline]
	fn add_assign(&mut self, rhs: &Self) {
		AddAssign::add_assign(self, *rhs);
	}
}

impl AddAssign<isize> for Offset {
	#[inline]
	fn add_assign(&mut self, rhs: isize) {
		AddAssign::add_assign(&mut self.0, rhs);
	}
}

impl AddAssign<&isize> for Offset {
	#[inline]
	fn add_assign(&mut self, rhs: &isize) {
		AddAssign::add_assign(self, *rhs);
	}
}

impl CheckedAdd for Offset {
	type Output = Self;

	fn checked_add(self, rhs: Self) -> Option<Self::Output> {
		Some(Self(CheckedAdd::checked_add(self.0, rhs.0)?))
	}
}

impl CheckedAdd<&Self> for Offset {
	type Output = Self;

	fn checked_add(self, rhs: &Self) -> Option<Self::Output> {
		CheckedAdd::checked_add(self, *rhs)
	}
}

impl CheckedAdd<Offset> for &Offset {
	type Output = Offset;

	fn checked_add(self, rhs: Offset) -> Option<Self::Output> {
		CheckedAdd::checked_add(*self, rhs)
	}
}

impl CheckedAdd for &Offset {
	type Output = Offset;

	fn checked_add(self, rhs: Self) -> Option<Self::Output> {
		CheckedAdd::checked_add(*self, *rhs)
	}
}

impl CheckedAdd<isize> for Offset {
	type Output = Self;

	fn checked_add(self, rhs: isize) -> Option<Self::Output> {
		Some(Self(CheckedAdd::checked_add(self.0, rhs)?))
	}
}

impl CheckedAdd<&isize> for Offset {
	type Output = Self;

	fn checked_add(self, rhs: &isize) -> Option<Self::Output> {
		CheckedAdd::checked_add(self, *rhs)
	}
}

impl CheckedAdd<isize> for &Offset {
	type Output = Offset;

	fn checked_add(self, rhs: isize) -> Option<Self::Output> {
		CheckedAdd::checked_add(*self, rhs)
	}
}

impl CheckedAdd<&isize> for &Offset {
	type Output = Offset;

	fn checked_add(self, rhs: &isize) -> Option<Self::Output> {
		CheckedAdd::checked_add(*self, *rhs)
	}
}

impl CheckedAddAssign for Offset {
	fn checked_add_assign(&mut self, rhs: Self) {
		CheckedAddAssign::checked_add_assign(&mut self.0, rhs.0);
	}
}

impl CheckedAddAssign<&Self> for Offset {
	fn checked_add_assign(&mut self, rhs: &Self) {
		CheckedAddAssign::checked_add_assign(self, *rhs);
	}
}

impl CheckedAddAssign<isize> for Offset {
	fn checked_add_assign(&mut self, rhs: isize) {
		CheckedAddAssign::checked_add_assign(&mut self.0, rhs);
	}
}

impl CheckedAddAssign<&isize> for Offset {
	fn checked_add_assign(&mut self, rhs: &isize) {
		CheckedAddAssign::checked_add_assign(self, *rhs);
	}
}

impl CheckedDiv for Offset {
	type Output = Self;

	fn checked_div(self, rhs: Self) -> Option<Self::Output> {
		Some(Self(CheckedDiv::checked_div(self.0, rhs.0)?))
	}
}

impl CheckedDiv<&Self> for Offset {
	type Output = Self;

	fn checked_div(self, rhs: &Self) -> Option<Self::Output> {
		CheckedDiv::checked_div(self, *rhs)
	}
}

impl CheckedDiv<Offset> for &Offset {
	type Output = Offset;

	fn checked_div(self, rhs: Offset) -> Option<Self::Output> {
		CheckedDiv::checked_div(*self, rhs)
	}
}

impl CheckedDiv for &Offset {
	type Output = Offset;

	fn checked_div(self, rhs: Self) -> Option<Self::Output> {
		CheckedDiv::checked_div(*self, *rhs)
	}
}

impl CheckedDiv<isize> for Offset {
	type Output = Self;

	fn checked_div(self, rhs: isize) -> Option<Self::Output> {
		Some(Self(CheckedDiv::checked_div(self.0, rhs)?))
	}
}

impl CheckedDiv<&isize> for Offset {
	type Output = Self;

	fn checked_div(self, rhs: &isize) -> Option<Self::Output> {
		CheckedDiv::checked_div(self, *rhs)
	}
}

impl CheckedDiv<isize> for &Offset {
	type Output = Offset;

	fn checked_div(self, rhs: isize) -> Option<Self::Output> {
		CheckedDiv::checked_div(*self, rhs)
	}
}

impl CheckedDiv<&isize> for &Offset {
	type Output = Offset;

	fn checked_div(self, rhs: &isize) -> Option<Self::Output> {
		CheckedDiv::checked_div(*self, *rhs)
	}
}

impl CheckedDivAssign for Offset {
	fn checked_div_assign(&mut self, rhs: Self) {
		CheckedDivAssign::checked_div_assign(&mut self.0, rhs.0);
	}
}

impl CheckedDivAssign<&Self> for Offset {
	fn checked_div_assign(&mut self, rhs: &Self) {
		CheckedDivAssign::checked_div_assign(self, *rhs);
	}
}

impl CheckedDivAssign<isize> for Offset {
	fn checked_div_assign(&mut self, rhs: isize) {
		CheckedDivAssign::checked_div_assign(&mut self.0, rhs);
	}
}

impl CheckedDivAssign<&isize> for Offset {
	fn checked_div_assign(&mut self, rhs: &isize) {
		CheckedDivAssign::checked_div_assign(self, *rhs);
	}
}

impl CheckedMul for Offset {
	type Output = Self;

	fn checked_mul(self, rhs: Self) -> Option<Self::Output> {
		Some(Self(CheckedMul::checked_mul(self.0, rhs.0)?))
	}
}

impl CheckedMul<&Self> for Offset {
	type Output = Self;

	fn checked_mul(self, rhs: &Self) -> Option<Self::Output> {
		CheckedMul::checked_mul(self, *rhs)
	}
}

impl CheckedMul<Offset> for &Offset {
	type Output = Offset;

	fn checked_mul(self, rhs: Offset) -> Option<Self::Output> {
		CheckedMul::checked_mul(*self, rhs)
	}
}

impl CheckedMul for &Offset {
	type Output = Offset;

	fn checked_mul(self, rhs: Self) -> Option<Self::Output> {
		CheckedMul::checked_mul(*self, *rhs)
	}
}

impl CheckedMul<isize> for Offset {
	type Output = Self;

	fn checked_mul(self, rhs: isize) -> Option<Self::Output> {
		Some(Self(CheckedMul::checked_mul(self.0, rhs)?))
	}
}

impl CheckedMul<&isize> for Offset {
	type Output = Self;

	fn checked_mul(self, rhs: &isize) -> Option<Self::Output> {
		CheckedMul::checked_mul(self, *rhs)
	}
}

impl CheckedMul<isize> for &Offset {
	type Output = Offset;

	fn checked_mul(self, rhs: isize) -> Option<Self::Output> {
		CheckedMul::checked_mul(*self, rhs)
	}
}

impl CheckedMul<&isize> for &Offset {
	type Output = Offset;

	fn checked_mul(self, rhs: &isize) -> Option<Self::Output> {
		CheckedMul::checked_mul(*self, *rhs)
	}
}

impl CheckedMulAssign for Offset {
	fn checked_mul_assign(&mut self, rhs: Self) {
		CheckedMulAssign::checked_mul_assign(&mut self.0, rhs.0);
	}
}

impl CheckedMulAssign<&Self> for Offset {
	fn checked_mul_assign(&mut self, rhs: &Self) {
		CheckedMulAssign::checked_mul_assign(self, *rhs);
	}
}

impl CheckedMulAssign<isize> for Offset {
	fn checked_mul_assign(&mut self, rhs: isize) {
		CheckedMulAssign::checked_mul_assign(&mut self.0, rhs);
	}
}

impl CheckedMulAssign<&isize> for Offset {
	fn checked_mul_assign(&mut self, rhs: &isize) {
		CheckedMulAssign::checked_mul_assign(self, *rhs);
	}
}

impl CheckedNeg for Offset {
	type Output = Self;

	fn checked_neg(self) -> Option<Self::Output> {
		Some(Self(CheckedNeg::checked_neg(self.0)?))
	}
}

impl CheckedNeg for &Offset {
	type Output = Offset;

	fn checked_neg(self) -> Option<Self::Output> {
		CheckedNeg::checked_neg(*self)
	}
}

impl CheckedRem for Offset {
	type Output = Self;

	fn checked_rem(self, rhs: Self) -> Option<Self::Output> {
		Some(Self(CheckedRem::checked_rem(self.0, rhs.0)?))
	}
}

impl CheckedRem<&Self> for Offset {
	type Output = Self;

	fn checked_rem(self, rhs: &Self) -> Option<Self::Output> {
		CheckedRem::checked_rem(self, *rhs)
	}
}

impl CheckedRem<Offset> for &Offset {
	type Output = Offset;

	fn checked_rem(self, rhs: Offset) -> Option<Self::Output> {
		CheckedRem::checked_rem(*self, rhs)
	}
}

impl CheckedRem for &Offset {
	type Output = Offset;

	fn checked_rem(self, rhs: Self) -> Option<Self::Output> {
		CheckedRem::checked_rem(*self, *rhs)
	}
}

impl CheckedRem<isize> for Offset {
	type Output = Self;

	fn checked_rem(self, rhs: isize) -> Option<Self::Output> {
		Some(Self(CheckedRem::checked_rem(self.0, rhs)?))
	}
}

impl CheckedRem<&isize> for Offset {
	type Output = Self;

	fn checked_rem(self, rhs: &isize) -> Option<Self::Output> {
		CheckedRem::checked_rem(self, *rhs)
	}
}

impl CheckedRem<isize> for &Offset {
	type Output = Offset;

	fn checked_rem(self, rhs: isize) -> Option<Self::Output> {
		CheckedRem::checked_rem(*self, rhs)
	}
}

impl CheckedRem<&isize> for &Offset {
	type Output = Offset;

	fn checked_rem(self, rhs: &isize) -> Option<Self::Output> {
		CheckedRem::checked_rem(*self, *rhs)
	}
}

impl CheckedRemAssign for Offset {
	fn checked_rem_assign(&mut self, rhs: Self) {
		CheckedRemAssign::checked_rem_assign(&mut self.0, rhs.0);
	}
}

impl CheckedRemAssign<&Self> for Offset {
	fn checked_rem_assign(&mut self, rhs: &Self) {
		CheckedRemAssign::checked_rem_assign(self, *rhs);
	}
}

impl CheckedRemAssign<isize> for Offset {
	fn checked_rem_assign(&mut self, rhs: isize) {
		CheckedRemAssign::checked_rem_assign(&mut self.0, rhs);
	}
}

impl CheckedRemAssign<&isize> for Offset {
	fn checked_rem_assign(&mut self, rhs: &isize) {
		CheckedRemAssign::checked_rem_assign(self, *rhs);
	}
}

impl CheckedSub for Offset {
	type Output = Self;

	fn checked_sub(self, rhs: Self) -> Option<Self::Output> {
		Some(Self(CheckedSub::checked_sub(self.0, rhs.0)?))
	}
}

impl CheckedSub<&Self> for Offset {
	type Output = Self;

	fn checked_sub(self, rhs: &Self) -> Option<Self::Output> {
		CheckedSub::checked_sub(self, *rhs)
	}
}

impl CheckedSub<Offset> for &Offset {
	type Output = Offset;

	fn checked_sub(self, rhs: Offset) -> Option<Self::Output> {
		CheckedSub::checked_sub(*self, rhs)
	}
}

impl CheckedSub for &Offset {
	type Output = Offset;

	fn checked_sub(self, rhs: Self) -> Option<Self::Output> {
		CheckedSub::checked_sub(*self, *rhs)
	}
}

impl CheckedSub<isize> for Offset {
	type Output = Self;

	fn checked_sub(self, rhs: isize) -> Option<Self::Output> {
		Some(Self(CheckedSub::checked_sub(self.0, rhs)?))
	}
}

impl CheckedSub<&isize> for Offset {
	type Output = Self;

	fn checked_sub(self, rhs: &isize) -> Option<Self::Output> {
		CheckedSub::checked_sub(self, *rhs)
	}
}

impl CheckedSub<isize> for &Offset {
	type Output = Offset;

	fn checked_sub(self, rhs: isize) -> Option<Self::Output> {
		CheckedSub::checked_sub(*self, rhs)
	}
}

impl CheckedSub<&isize> for &Offset {
	type Output = Offset;

	fn checked_sub(self, rhs: &isize) -> Option<Self::Output> {
		CheckedSub::checked_sub(*self, *rhs)
	}
}

impl CheckedSubAssign for Offset {
	fn checked_sub_assign(&mut self, rhs: Self) {
		CheckedSubAssign::checked_sub_assign(&mut self.0, rhs.0);
	}
}

impl CheckedSubAssign<&Self> for Offset {
	fn checked_sub_assign(&mut self, rhs: &Self) {
		CheckedSubAssign::checked_sub_assign(self, *rhs);
	}
}

impl CheckedSubAssign<isize> for Offset {
	fn checked_sub_assign(&mut self, rhs: isize) {
		CheckedSubAssign::checked_sub_assign(&mut self.0, rhs);
	}
}

impl CheckedSubAssign<&isize> for Offset {
	fn checked_sub_assign(&mut self, rhs: &isize) {
		CheckedSubAssign::checked_sub_assign(self, *rhs);
	}
}

impl Div for Offset {
	type Output = Self;

	#[inline]
	fn div(self, rhs: Self) -> Self::Output {
		Self(Div::div(self.0, rhs.0))
	}
}

impl Div<&Self> for Offset {
	type Output = Self;

	#[inline]
	fn div(self, rhs: &Self) -> Self::Output {
		Div::div(self, *rhs)
	}
}

impl Div<Offset> for &Offset {
	type Output = Offset;

	#[inline]
	fn div(self, rhs: Offset) -> Self::Output {
		Div::div(*self, rhs)
	}
}

impl Div for &Offset {
	type Output = Offset;

	#[inline]
	fn div(self, rhs: Self) -> Self::Output {
		Div::div(*self, *rhs)
	}
}

impl Div<isize> for Offset {
	type Output = Self;

	#[inline]
	fn div(self, rhs: isize) -> Self::Output {
		Self(Div::div(self.0, rhs))
	}
}

impl Div<&isize> for Offset {
	type Output = Self;

	#[inline]
	fn div(self, rhs: &isize) -> Self::Output {
		Div::div(self, *rhs)
	}
}

impl Div<isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn div(self, rhs: isize) -> Self::Output {
		Div::div(*self, rhs)
	}
}

impl Div<&isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn div(self, rhs: &isize) -> Self::Output {
		Div::div(*self, *rhs)
	}
}

impl DivAssign for Offset {
	#[inline]
	fn div_assign(&mut self, rhs: Self) {
		DivAssign::div_assign(&mut self.0, rhs.0);
	}
}

impl DivAssign<&Self> for Offset {
	#[inline]
	fn div_assign(&mut self, rhs: &Self) {
		DivAssign::div_assign(self, *rhs);
	}
}

impl DivAssign<isize> for Offset {
	#[inline]
	fn div_assign(&mut self, rhs: isize) {
		DivAssign::div_assign(&mut self.0, rhs);
	}
}

impl DivAssign<&isize> for Offset {
	#[inline]
	fn div_assign(&mut self, rhs: &isize) {
		DivAssign::div_assign(self, *rhs);
	}
}

impl Mul for Offset {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Self(Mul::mul(self.0, rhs.0))
	}
}

impl Mul<&Self> for Offset {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: &Self) -> Self::Output {
		Mul::mul(self, *rhs)
	}
}

impl Mul<Offset> for &Offset {
	type Output = Offset;

	#[inline]
	fn mul(self, rhs: Offset) -> Self::Output {
		Mul::mul(*self, rhs)
	}
}

impl Mul for &Offset {
	type Output = Offset;

	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Mul::mul(*self, *rhs)
	}
}

impl Mul<isize> for Offset {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: isize) -> Self::Output {
		Self(Mul::mul(self.0, rhs))
	}
}

impl Mul<&isize> for Offset {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: &isize) -> Self::Output {
		Mul::mul(self, *rhs)
	}
}

impl Mul<isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn mul(self, rhs: isize) -> Self::Output {
		Mul::mul(*self, rhs)
	}
}

impl Mul<&isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn mul(self, rhs: &isize) -> Self::Output {
		Mul::mul(*self, *rhs)
	}
}

impl MulAssign for Offset {
	#[inline]
	fn mul_assign(&mut self, rhs: Self) {
		MulAssign::mul_assign(&mut self.0, rhs.0);
	}
}

impl MulAssign<&Self> for Offset {
	#[inline]
	fn mul_assign(&mut self, rhs: &Self) {
		MulAssign::mul_assign(self, *rhs);
	}
}

impl MulAssign<isize> for Offset {
	#[inline]
	fn mul_assign(&mut self, rhs: isize) {
		MulAssign::mul_assign(&mut self.0, rhs);
	}
}

impl MulAssign<&isize> for Offset {
	#[inline]
	fn mul_assign(&mut self, rhs: &isize) {
		MulAssign::mul_assign(self, *rhs);
	}
}

impl Neg for Offset {
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		Self(Neg::neg(self.0))
	}
}

impl Neg for &Offset {
	type Output = Offset;

	#[inline]
	fn neg(self) -> Self::Output {
		Neg::neg(*self)
	}
}

impl Not for Offset {
	type Output = Self;

	#[inline]
	fn not(self) -> Self::Output {
		Self(Not::not(self.0))
	}
}

impl Not for &Offset {
	type Output = Offset;

	#[inline]
	fn not(self) -> Self::Output {
		Not::not(*self)
	}
}

impl Rem for Offset {
	type Output = Self;

	#[inline]
	fn rem(self, rhs: Self) -> Self::Output {
		Self(Rem::rem(self.0, rhs.0))
	}
}

impl Rem<&Self> for Offset {
	type Output = Self;

	#[inline]
	fn rem(self, rhs: &Self) -> Self::Output {
		Rem::rem(self, *rhs)
	}
}

impl Rem<Offset> for &Offset {
	type Output = Offset;

	#[inline]
	fn rem(self, rhs: Offset) -> Self::Output {
		Rem::rem(*self, rhs)
	}
}

impl Rem for &Offset {
	type Output = Offset;

	#[inline]
	fn rem(self, rhs: Self) -> Self::Output {
		Rem::rem(*self, *rhs)
	}
}

impl Rem<isize> for Offset {
	type Output = Self;

	#[inline]
	fn rem(self, rhs: isize) -> Self::Output {
		Self(Rem::rem(self.0, rhs))
	}
}

impl Rem<&isize> for Offset {
	type Output = Self;

	#[inline]
	fn rem(self, rhs: &isize) -> Self::Output {
		Rem::rem(self, *rhs)
	}
}

impl Rem<isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn rem(self, rhs: isize) -> Self::Output {
		Rem::rem(*self, rhs)
	}
}

impl Rem<&isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn rem(self, rhs: &isize) -> Self::Output {
		Rem::rem(*self, *rhs)
	}
}

impl RemAssign for Offset {
	#[inline]
	fn rem_assign(&mut self, rhs: Self) {
		RemAssign::rem_assign(&mut self.0, rhs.0);
	}
}

impl RemAssign<&Self> for Offset {
	#[inline]
	fn rem_assign(&mut self, rhs: &Self) {
		RemAssign::rem_assign(self, *rhs);
	}
}

impl RemAssign<isize> for Offset {
	#[inline]
	fn rem_assign(&mut self, rhs: isize) {
		RemAssign::rem_assign(&mut self.0, rhs);
	}
}

impl RemAssign<&isize> for Offset {
	#[inline]
	fn rem_assign(&mut self, rhs: &isize) {
		RemAssign::rem_assign(self, *rhs);
	}
}

impl SaturatingAdd for Offset {
	type Output = Self;

	fn saturating_add(self, rhs: Self) -> Self::Output {
		Self(SaturatingAdd::saturating_add(self.0, rhs.0))
	}
}

impl SaturatingAdd<&Self> for Offset {
	type Output = Self;

	fn saturating_add(self, rhs: &Self) -> Self::Output {
		SaturatingAdd::saturating_add(self, *rhs)
	}
}

impl SaturatingAdd<Offset> for &Offset {
	type Output = Offset;

	fn saturating_add(self, rhs: Offset) -> Self::Output {
		SaturatingAdd::saturating_add(*self, rhs)
	}
}

impl SaturatingAdd for &Offset {
	type Output = Offset;

	fn saturating_add(self, rhs: Self) -> Self::Output {
		SaturatingAdd::saturating_add(*self, *rhs)
	}
}

impl SaturatingAdd<isize> for Offset {
	type Output = Self;

	fn saturating_add(self, rhs: isize) -> Self::Output {
		Self(SaturatingAdd::saturating_add(self.0, rhs))
	}
}

impl SaturatingAdd<&isize> for Offset {
	type Output = Self;

	fn saturating_add(self, rhs: &isize) -> Self::Output {
		SaturatingAdd::saturating_add(self, *rhs)
	}
}

impl SaturatingAdd<isize> for &Offset {
	type Output = Offset;

	fn saturating_add(self, rhs: isize) -> Self::Output {
		SaturatingAdd::saturating_add(*self, rhs)
	}
}

impl SaturatingAdd<&isize> for &Offset {
	type Output = Offset;

	fn saturating_add(self, rhs: &isize) -> Self::Output {
		SaturatingAdd::saturating_add(*self, *rhs)
	}
}

impl SaturatingAddAssign for Offset {
	fn saturating_add_assign(&mut self, rhs: Self) {
		SaturatingAddAssign::saturating_add_assign(&mut self.0, rhs.0);
	}
}

impl SaturatingAddAssign<&Self> for Offset {
	fn saturating_add_assign(&mut self, rhs: &Self) {
		SaturatingAddAssign::saturating_add_assign(self, *rhs);
	}
}

impl SaturatingAddAssign<isize> for Offset {
	fn saturating_add_assign(&mut self, rhs: isize) {
		SaturatingAddAssign::saturating_add_assign(&mut self.0, rhs);
	}
}

impl SaturatingAddAssign<&isize> for Offset {
	fn saturating_add_assign(&mut self, rhs: &isize) {
		SaturatingAddAssign::saturating_add_assign(self, *rhs);
	}
}

impl SaturatingDiv for Offset {
	type Output = Self;

	fn saturating_div(self, rhs: Self) -> Self::Output {
		Self(SaturatingDiv::saturating_div(self.0, rhs.0))
	}
}

impl SaturatingDiv<&Self> for Offset {
	type Output = Self;

	fn saturating_div(self, rhs: &Self) -> Self::Output {
		SaturatingDiv::saturating_div(self, *rhs)
	}
}

impl SaturatingDiv<Offset> for &Offset {
	type Output = Offset;

	fn saturating_div(self, rhs: Offset) -> Self::Output {
		SaturatingDiv::saturating_div(*self, rhs)
	}
}

impl SaturatingDiv for &Offset {
	type Output = Offset;

	fn saturating_div(self, rhs: Self) -> Self::Output {
		SaturatingDiv::saturating_div(*self, *rhs)
	}
}

impl SaturatingDiv<isize> for Offset {
	type Output = Self;

	fn saturating_div(self, rhs: isize) -> Self::Output {
		Self(SaturatingDiv::saturating_div(self.0, rhs))
	}
}

impl SaturatingDiv<&isize> for Offset {
	type Output = Self;

	fn saturating_div(self, rhs: &isize) -> Self::Output {
		SaturatingDiv::saturating_div(self, *rhs)
	}
}

impl SaturatingDiv<isize> for &Offset {
	type Output = Offset;

	fn saturating_div(self, rhs: isize) -> Self::Output {
		SaturatingDiv::saturating_div(*self, rhs)
	}
}

impl SaturatingDiv<&isize> for &Offset {
	type Output = Offset;

	fn saturating_div(self, rhs: &isize) -> Self::Output {
		SaturatingDiv::saturating_div(*self, *rhs)
	}
}

impl SaturatingDivAssign for Offset {
	fn saturating_div_assign(&mut self, rhs: Self) {
		SaturatingDivAssign::saturating_div_assign(&mut self.0, rhs.0);
	}
}

impl SaturatingDivAssign<&Self> for Offset {
	fn saturating_div_assign(&mut self, rhs: &Self) {
		SaturatingDivAssign::saturating_div_assign(self, *rhs);
	}
}

impl SaturatingDivAssign<isize> for Offset {
	fn saturating_div_assign(&mut self, rhs: isize) {
		SaturatingDivAssign::saturating_div_assign(&mut self.0, rhs);
	}
}

impl SaturatingDivAssign<&isize> for Offset {
	fn saturating_div_assign(&mut self, rhs: &isize) {
		SaturatingDivAssign::saturating_div_assign(self, *rhs);
	}
}

impl SaturatingMul for Offset {
	type Output = Self;

	fn saturating_mul(self, rhs: Self) -> Self::Output {
		Self(SaturatingMul::saturating_mul(self.0, rhs.0))
	}
}

impl SaturatingMul<&Self> for Offset {
	type Output = Self;

	fn saturating_mul(self, rhs: &Self) -> Self::Output {
		SaturatingMul::saturating_mul(self, *rhs)
	}
}

impl SaturatingMul<Offset> for &Offset {
	type Output = Offset;

	fn saturating_mul(self, rhs: Offset) -> Self::Output {
		SaturatingMul::saturating_mul(*self, rhs)
	}
}

impl SaturatingMul for &Offset {
	type Output = Offset;

	fn saturating_mul(self, rhs: Self) -> Self::Output {
		SaturatingMul::saturating_mul(*self, *rhs)
	}
}

impl Sub for Offset {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		Self(Sub::sub(self.0, rhs.0))
	}
}

impl Sub<&Self> for Offset {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: &Self) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<Offset> for &Offset {
	type Output = Offset;

	#[inline]
	fn sub(self, rhs: Offset) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub for &Offset {
	type Output = Offset;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}

impl Sub<isize> for Offset {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: isize) -> Self::Output {
		Self(Sub::sub(self.0, rhs))
	}
}

impl Sub<&isize> for Offset {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: &isize) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn sub(self, rhs: isize) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub<&isize> for &Offset {
	type Output = Offset;

	#[inline]
	fn sub(self, rhs: &isize) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}

impl SubAssign for Offset {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		SubAssign::sub_assign(&mut self.0, rhs.0);
	}
}

impl SubAssign<&Self> for Offset {
	#[inline]
	fn sub_assign(&mut self, rhs: &Self) {
		SubAssign::sub_assign(self, *rhs);
	}
}

impl SubAssign<isize> for Offset {
	#[inline]
	fn sub_assign(&mut self, rhs: isize) {
		SubAssign::sub_assign(&mut self.0, rhs);
	}
}

impl SubAssign<&isize> for Offset {
	#[inline]
	fn sub_assign(&mut self, rhs: &isize) {
		SubAssign::sub_assign(self, *rhs);
	}
}

impl WrappingAdd for Offset {
	type Output = Self;

	#[inline]
	fn wrapping_add(self, rhs: Self) -> Self::Output {
		Self(WrappingAdd::wrapping_add(self.0, rhs.0))
	}
}

impl WrappingAdd<&Self> for Offset {
	type Output = Self;

	#[inline]
	fn wrapping_add(self, rhs: &Self) -> Self::Output {
		WrappingAdd::wrapping_add(self, *rhs)
	}
}

impl WrappingAdd<Offset> for &Offset {
	type Output = Offset;

	#[inline]
	fn wrapping_add(self, rhs: Offset) -> Self::Output {
		WrappingAdd::wrapping_add(*self, rhs)
	}
}

impl WrappingAdd for &Offset {
	type Output = Offset;

	#[inline]
	fn wrapping_add(self, rhs: Self) -> Self::Output {
		WrappingAdd::wrapping_add(*self, *rhs)
	}
}
