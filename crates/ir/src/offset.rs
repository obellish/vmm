use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::{
		Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign,
	},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vmm_span::Walk;
use vmm_utils::GetOrZero;
use vmm_wrap::ops::{
	WrappingAdd, WrappingAddAssign, WrappingDiv, WrappingDivAssign, WrappingMul, WrappingNeg,
	WrappingSub, WrappingSubAssign,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Offset(pub isize);

impl Offset {
	#[must_use]
	pub const fn abs(self) -> Self {
		Self(self.0.abs())
	}

	#[must_use]
	pub const fn value(self) -> isize {
		self.0
	}

	#[must_use]
	pub const fn new(value: isize) -> Self {
		Self(value)
	}
}

impl Add for Offset {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(Add::add(self.0, rhs.0))
	}
}

impl Add<&Self> for Offset {
	type Output = Self;

	fn add(self, rhs: &Self) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<Offset> for &Offset {
	type Output = Offset;

	fn add(self, rhs: Offset) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add for &Offset {
	type Output = Offset;

	fn add(self, rhs: Self) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl Add<isize> for Offset {
	type Output = Self;

	fn add(self, rhs: isize) -> Self::Output {
		Self(Add::add(self.0, rhs))
	}
}

impl Add<&isize> for Offset {
	type Output = Self;

	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<isize> for &Offset {
	type Output = Offset;

	fn add(self, rhs: isize) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add<&isize> for &Offset {
	type Output = Offset;

	fn add(self, rhs: &isize) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl AddAssign for Offset {
	fn add_assign(&mut self, rhs: Self) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&Self> for Offset {
	fn add_assign(&mut self, rhs: &Self) {
		*self = Add::add(*self, *rhs);
	}
}

impl AddAssign<isize> for Offset {
	fn add_assign(&mut self, rhs: isize) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&isize> for Offset {
	fn add_assign(&mut self, rhs: &isize) {
		*self = Add::add(*self, *rhs);
	}
}

impl<'de> Deserialize<'de> for Offset {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		isize::deserialize(deserializer).map(Self::new)
	}
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let alt = f.alternate();

		if alt {
			f.write_char('[')?;
		}

		Display::fmt(&self.value(), f)?;

		if alt {
			f.write_char(']')?;
		}

		Ok(())
	}
}

impl Div for Offset {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self(Div::div(self.0, rhs.0))
	}
}

impl Div<&Self> for Offset {
	type Output = Self;

	fn div(self, rhs: &Self) -> Self::Output {
		Div::div(self, *rhs)
	}
}

impl Div<Offset> for &Offset {
	type Output = Offset;

	fn div(self, rhs: Offset) -> Self::Output {
		Div::div(*self, rhs)
	}
}

impl Div for &Offset {
	type Output = Offset;

	fn div(self, rhs: Self) -> Self::Output {
		Div::div(*self, *rhs)
	}
}

impl Div<isize> for Offset {
	type Output = Self;

	fn div(self, rhs: isize) -> Self::Output {
		Self(Div::div(self.0, rhs))
	}
}

impl Div<&isize> for Offset {
	type Output = Self;

	fn div(self, rhs: &isize) -> Self::Output {
		Div::div(self, *rhs)
	}
}

impl Div<isize> for &Offset {
	type Output = Offset;

	fn div(self, rhs: isize) -> Self::Output {
		Div::div(*self, rhs)
	}
}

impl Div<&isize> for &Offset {
	type Output = Offset;

	fn div(self, rhs: &isize) -> Self::Output {
		Div::div(*self, *rhs)
	}
}

impl DivAssign for Offset {
	fn div_assign(&mut self, rhs: Self) {
		*self = Div::div(*self, rhs);
	}
}

impl DivAssign<&Self> for Offset {
	fn div_assign(&mut self, rhs: &Self) {
		*self = Div::div(*self, *rhs);
	}
}

impl DivAssign<isize> for Offset {
	fn div_assign(&mut self, rhs: isize) {
		*self = Div::div(*self, rhs);
	}
}

impl DivAssign<&isize> for Offset {
	fn div_assign(&mut self, rhs: &isize) {
		*self = Div::div(*self, *rhs);
	}
}

impl From<&Self> for Offset {
	fn from(value: &Self) -> Self {
		*value
	}
}

impl From<isize> for Offset {
	fn from(value: isize) -> Self {
		Self::new(value)
	}
}

impl From<&isize> for Offset {
	fn from(value: &isize) -> Self {
		(*value).into()
	}
}

impl GetOrZero<Self> for Offset {
	fn get_or_zero(self) -> Self {
		self
	}
}

impl GetOrZero<Offset> for Option<Offset> {
	fn get_or_zero(self) -> Offset {
		match self {
			Some(offset) => offset,
			None => Offset::new(0),
		}
	}
}

impl Mul for Offset {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self(Mul::mul(self.0, rhs.0))
	}
}

impl Mul<&Self> for Offset {
	type Output = Self;

	fn mul(self, rhs: &Self) -> Self::Output {
		Mul::mul(self, *rhs)
	}
}

impl Mul<Offset> for &Offset {
	type Output = Offset;

	fn mul(self, rhs: Offset) -> Self::Output {
		Mul::mul(*self, rhs)
	}
}

impl Mul for &Offset {
	type Output = Offset;

	fn mul(self, rhs: Self) -> Self::Output {
		Mul::mul(*self, *rhs)
	}
}

impl Mul<isize> for Offset {
	type Output = Self;

	fn mul(self, rhs: isize) -> Self::Output {
		Self(Mul::mul(self.0, rhs))
	}
}

impl Mul<&isize> for Offset {
	type Output = Self;

	fn mul(self, rhs: &isize) -> Self::Output {
		Mul::mul(self, *rhs)
	}
}

impl Mul<isize> for &Offset {
	type Output = Offset;

	fn mul(self, rhs: isize) -> Self::Output {
		Mul::mul(*self, rhs)
	}
}

impl Mul<&isize> for &Offset {
	type Output = Offset;

	fn mul(self, rhs: &isize) -> Self::Output {
		Mul::mul(*self, *rhs)
	}
}

impl MulAssign for Offset {
	fn mul_assign(&mut self, rhs: Self) {
		*self = Mul::mul(*self, rhs);
	}
}

impl MulAssign<&Self> for Offset {
	fn mul_assign(&mut self, rhs: &Self) {
		*self = Mul::mul(*self, *rhs);
	}
}

impl MulAssign<isize> for Offset {
	fn mul_assign(&mut self, rhs: isize) {
		*self = Mul::mul(*self, rhs);
	}
}

impl MulAssign<&isize> for Offset {
	fn mul_assign(&mut self, rhs: &isize) {
		*self = Mul::mul(*self, *rhs);
	}
}

impl Neg for Offset {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self(Neg::neg(self.0))
	}
}

impl Neg for &Offset {
	type Output = Offset;

	fn neg(self) -> Self::Output {
		Neg::neg(*self)
	}
}

impl Not for Offset {
	type Output = Self;

	fn not(self) -> Self::Output {
		Self(Not::not(self.0))
	}
}

impl Not for &Offset {
	type Output = Offset;

	fn not(self) -> Self::Output {
		Not::not(*self)
	}
}

impl PartialEq<isize> for Offset {
	fn eq(&self, other: &isize) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl PartialEq<Offset> for isize {
	fn eq(&self, other: &Offset) -> bool {
		PartialEq::eq(self, &other.0)
	}
}

impl PartialOrd<isize> for Offset {
	fn partial_cmp(&self, other: &isize) -> Option<Ordering> {
		Some(self.0.cmp(other))
	}
}

impl PartialOrd<Offset> for isize {
	fn partial_cmp(&self, other: &Offset) -> Option<Ordering> {
		Some(self.cmp(&other.0))
	}
}

impl Rem for Offset {
	type Output = Self;

	fn rem(self, rhs: Self) -> Self::Output {
		Self(Rem::rem(self.0, rhs.0))
	}
}

impl Rem<&Self> for Offset {
	type Output = Self;

	fn rem(self, rhs: &Self) -> Self::Output {
		Rem::rem(self, *rhs)
	}
}

impl Rem<Offset> for &Offset {
	type Output = Offset;

	fn rem(self, rhs: Offset) -> Self::Output {
		Rem::rem(*self, rhs)
	}
}

impl Rem for &Offset {
	type Output = Offset;

	fn rem(self, rhs: Self) -> Self::Output {
		Rem::rem(*self, *rhs)
	}
}

impl Rem<isize> for Offset {
	type Output = Self;

	fn rem(self, rhs: isize) -> Self::Output {
		Self(Rem::rem(self.0, rhs))
	}
}

impl Rem<&isize> for Offset {
	type Output = Self;

	fn rem(self, rhs: &isize) -> Self::Output {
		Rem::rem(self, *rhs)
	}
}

impl Rem<isize> for &Offset {
	type Output = Offset;

	fn rem(self, rhs: isize) -> Self::Output {
		Rem::rem(*self, rhs)
	}
}

impl Rem<&isize> for &Offset {
	type Output = Offset;

	fn rem(self, rhs: &isize) -> Self::Output {
		Rem::rem(*self, *rhs)
	}
}

impl RemAssign for Offset {
	fn rem_assign(&mut self, rhs: Self) {
		*self = Rem::rem(*self, rhs);
	}
}

impl RemAssign<&Self> for Offset {
	fn rem_assign(&mut self, rhs: &Self) {
		*self = Rem::rem(*self, *rhs);
	}
}

impl RemAssign<isize> for Offset {
	fn rem_assign(&mut self, rhs: isize) {
		*self = Rem::rem(*self, rhs);
	}
}

impl RemAssign<&isize> for Offset {
	fn rem_assign(&mut self, rhs: &isize) {
		*self = Rem::rem(*self, *rhs);
	}
}

impl Serialize for Offset {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.0.serialize(serializer)
	}
}

impl Walk for Offset {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		isize::steps_between(&start.0, &end.0)
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		Some(Self(isize::forward_checked(start.0, count)?))
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		Some(Self(isize::backward_checked(start.0, count)?))
	}
}

impl Sub for Offset {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(Sub::sub(self.0, rhs.0))
	}
}

impl Sub<&Self> for Offset {
	type Output = Self;

	fn sub(self, rhs: &Self) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<Offset> for &Offset {
	type Output = Offset;

	fn sub(self, rhs: Offset) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub for &Offset {
	type Output = Offset;

	fn sub(self, rhs: Self) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}

impl Sub<isize> for Offset {
	type Output = Self;

	fn sub(self, rhs: isize) -> Self::Output {
		Self(Sub::sub(self.0, rhs))
	}
}

impl Sub<&isize> for Offset {
	type Output = Self;

	fn sub(self, rhs: &isize) -> Self::Output {
		Sub::sub(self, *rhs)
	}
}

impl Sub<isize> for &Offset {
	type Output = Offset;

	fn sub(self, rhs: isize) -> Self::Output {
		Sub::sub(*self, rhs)
	}
}

impl Sub<&isize> for &Offset {
	type Output = Offset;

	fn sub(self, rhs: &isize) -> Self::Output {
		Sub::sub(*self, *rhs)
	}
}

impl SubAssign for Offset {
	fn sub_assign(&mut self, rhs: Self) {
		*self = Sub::sub(*self, rhs);
	}
}

impl SubAssign<&Self> for Offset {
	fn sub_assign(&mut self, rhs: &Self) {
		*self = Sub::sub(*self, *rhs);
	}
}

impl SubAssign<isize> for Offset {
	fn sub_assign(&mut self, rhs: isize) {
		*self = Sub::sub(*self, rhs);
	}
}

impl SubAssign<&isize> for Offset {
	fn sub_assign(&mut self, rhs: &isize) {
		*self = Sub::sub(*self, *rhs);
	}
}

impl WrappingAdd for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		Self(WrappingAdd::wrapping_add(self.0, rhs.0))
	}
}

impl WrappingAdd<&Self> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: &Self) -> Self::Output {
		WrappingAdd::wrapping_add(self, *rhs)
	}
}

impl WrappingAdd<Offset> for &Offset {
	type Output = Offset;

	fn wrapping_add(self, rhs: Offset) -> Self::Output {
		WrappingAdd::wrapping_add(*self, rhs)
	}
}

impl WrappingAdd for &Offset {
	type Output = Offset;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		WrappingAdd::wrapping_add(*self, *rhs)
	}
}

impl WrappingAdd<isize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: isize) -> Self::Output {
		Self(WrappingAdd::wrapping_add(self.0, rhs))
	}
}

impl WrappingAdd<&isize> for Offset {
	type Output = Self;

	fn wrapping_add(self, rhs: &isize) -> Self::Output {
		WrappingAdd::wrapping_add(self, *rhs)
	}
}

impl WrappingAdd<isize> for &Offset {
	type Output = Offset;

	fn wrapping_add(self, rhs: isize) -> Self::Output {
		WrappingAdd::wrapping_add(*self, rhs)
	}
}

impl WrappingAdd<&isize> for &Offset {
	type Output = Offset;

	fn wrapping_add(self, rhs: &isize) -> Self::Output {
		WrappingAdd::wrapping_add(*self, *rhs)
	}
}

impl WrappingAddAssign for Offset {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = WrappingAdd::wrapping_add(*self, rhs);
	}
}

impl WrappingAddAssign<&Self> for Offset {
	fn wrapping_add_assign(&mut self, rhs: &Self) {
		*self = WrappingAdd::wrapping_add(*self, *rhs);
	}
}

impl WrappingAddAssign<isize> for Offset {
	fn wrapping_add_assign(&mut self, rhs: isize) {
		*self = WrappingAdd::wrapping_add(*self, rhs);
	}
}

impl WrappingAddAssign<&isize> for Offset {
	fn wrapping_add_assign(&mut self, rhs: &isize) {
		*self = WrappingAdd::wrapping_add(*self, *rhs);
	}
}

impl WrappingDiv for Offset {
	type Output = Self;

	fn wrapping_div(self, rhs: Self) -> Self::Output {
		Self(WrappingDiv::wrapping_div(self.0, rhs.0))
	}
}

impl WrappingDiv<&Self> for Offset {
	type Output = Self;

	fn wrapping_div(self, rhs: &Self) -> Self::Output {
		WrappingDiv::wrapping_div(self, *rhs)
	}
}

impl WrappingDiv<Offset> for &Offset {
	type Output = Offset;

	fn wrapping_div(self, rhs: Offset) -> Self::Output {
		WrappingDiv::wrapping_div(*self, rhs)
	}
}

impl WrappingDiv for &Offset {
	type Output = Offset;

	fn wrapping_div(self, rhs: Self) -> Self::Output {
		WrappingDiv::wrapping_div(*self, *rhs)
	}
}

impl WrappingDiv<isize> for Offset {
	type Output = Self;

	fn wrapping_div(self, rhs: isize) -> Self::Output {
		Self(WrappingDiv::wrapping_div(self.0, rhs))
	}
}

impl WrappingDiv<&isize> for Offset {
	type Output = Self;

	fn wrapping_div(self, rhs: &isize) -> Self::Output {
		WrappingDiv::wrapping_div(self, *rhs)
	}
}

impl WrappingDiv<isize> for &Offset {
	type Output = Offset;

	fn wrapping_div(self, rhs: isize) -> Self::Output {
		WrappingDiv::wrapping_div(*self, rhs)
	}
}

impl WrappingDiv<&isize> for &Offset {
	type Output = Offset;

	fn wrapping_div(self, rhs: &isize) -> Self::Output {
		WrappingDiv::wrapping_div(*self, *rhs)
	}
}

impl WrappingDivAssign for Offset {
	fn wrapping_div_assign(&mut self, rhs: Self) {
		*self = WrappingDiv::wrapping_div(*self, rhs);
	}
}

impl WrappingDivAssign<&Self> for Offset {
	fn wrapping_div_assign(&mut self, rhs: &Self) {
		*self = WrappingDiv::wrapping_div(*self, *rhs);
	}
}

impl WrappingDivAssign<isize> for Offset {
	fn wrapping_div_assign(&mut self, rhs: isize) {
		*self = WrappingDiv::wrapping_div(*self, rhs);
	}
}

impl WrappingDivAssign<&isize> for Offset {
	fn wrapping_div_assign(&mut self, rhs: &isize) {
		*self = WrappingDiv::wrapping_div(*self, *rhs);
	}
}

impl WrappingMul for Offset {
	type Output = Self;

	fn wrapping_mul(self, rhs: Self) -> Self::Output {
		Self(WrappingMul::wrapping_mul(self.0, rhs.0))
	}
}

impl WrappingMul<&Self> for Offset {
	type Output = Self;

	fn wrapping_mul(self, rhs: &Self) -> Self::Output {
		WrappingMul::wrapping_mul(self, *rhs)
	}
}

impl WrappingMul<Offset> for &Offset {
	type Output = Offset;

	fn wrapping_mul(self, rhs: Offset) -> Self::Output {
		WrappingMul::wrapping_mul(*self, rhs)
	}
}

impl WrappingMul for &Offset {
	type Output = Offset;

	fn wrapping_mul(self, rhs: Self) -> Self::Output {
		WrappingMul::wrapping_mul(*self, *rhs)
	}
}

impl WrappingMul<isize> for Offset {
	type Output = Self;

	fn wrapping_mul(self, rhs: isize) -> Self::Output {
		Self(WrappingMul::wrapping_mul(self.0, rhs))
	}
}

impl WrappingMul<&isize> for Offset {
	type Output = Self;

	fn wrapping_mul(self, rhs: &isize) -> Self::Output {
		WrappingMul::wrapping_mul(self, *rhs)
	}
}

impl WrappingMul<isize> for &Offset {
	type Output = Offset;

	fn wrapping_mul(self, rhs: isize) -> Self::Output {
		WrappingMul::wrapping_mul(*self, rhs)
	}
}

impl WrappingMul<&isize> for &Offset {
	type Output = Offset;

	fn wrapping_mul(self, rhs: &isize) -> Self::Output {
		WrappingMul::wrapping_mul(*self, *rhs)
	}
}

impl WrappingNeg for Offset {
	type Output = Self;

	fn wrapping_neg(self) -> Self::Output {
		Self(WrappingNeg::wrapping_neg(self.0))
	}
}

impl WrappingNeg for &Offset {
	type Output = Offset;

	fn wrapping_neg(self) -> Self::Output {
		WrappingNeg::wrapping_neg(*self)
	}
}

impl WrappingSub for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: Self) -> Self::Output {
		Self(WrappingSub::wrapping_sub(self.0, rhs.0))
	}
}

impl WrappingSub<&Self> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: &Self) -> Self::Output {
		WrappingSub::wrapping_sub(self, *rhs)
	}
}

impl WrappingSub<Offset> for &Offset {
	type Output = Offset;

	fn wrapping_sub(self, rhs: Offset) -> Self::Output {
		WrappingSub::wrapping_sub(*self, rhs)
	}
}

impl WrappingSub for &Offset {
	type Output = Offset;

	fn wrapping_sub(self, rhs: Self) -> Self::Output {
		WrappingSub::wrapping_sub(*self, *rhs)
	}
}

impl WrappingSub<isize> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: isize) -> Self::Output {
		Self(WrappingSub::wrapping_sub(self.0, rhs))
	}
}

impl WrappingSub<&isize> for Offset {
	type Output = Self;

	fn wrapping_sub(self, rhs: &isize) -> Self::Output {
		WrappingSub::wrapping_sub(self, *rhs)
	}
}

impl WrappingSub<isize> for &Offset {
	type Output = Offset;

	fn wrapping_sub(self, rhs: isize) -> Self::Output {
		WrappingSub::wrapping_sub(*self, rhs)
	}
}

impl WrappingSub<&isize> for &Offset {
	type Output = Offset;

	fn wrapping_sub(self, rhs: &isize) -> Self::Output {
		WrappingSub::wrapping_sub(*self, *rhs)
	}
}

impl WrappingSubAssign for Offset {
	fn wrapping_sub_assign(&mut self, rhs: Self) {
		*self = WrappingSub::wrapping_sub(*self, rhs);
	}
}

impl WrappingSubAssign<&Self> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: &Self) {
		*self = WrappingSub::wrapping_sub(*self, *rhs);
	}
}

impl WrappingSubAssign<isize> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: isize) {
		*self = WrappingSub::wrapping_sub(*self, rhs);
	}
}

impl WrappingSubAssign<&isize> for Offset {
	fn wrapping_sub_assign(&mut self, rhs: &isize) {
		*self = WrappingSub::wrapping_sub(*self, *rhs);
	}
}
