use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::{
		Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign,
	},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vmm_wrap::ops::{
	WrappingAdd, WrappingAddAssign, WrappingDiv, WrappingDivAssign, WrappingMul, WrappingNeg,
	WrappingSub, WrappingSubAssign,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[non_exhaustive]
pub enum Offset {
	Relative(isize),
}

impl Offset {
	#[must_use]
	pub const fn abs(self) -> Self {
		match self {
			Self::Relative(i) => Self::Relative(i.abs()),
		}
	}

	#[must_use]
	pub const fn is_relative(self) -> bool {
		matches!(self, Self::Relative(_))
	}
}

impl Add for Offset {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Add::add(lhs, rhs)),
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(Add::add(lhs, rhs)),
		}
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
		isize::deserialize(deserializer).map(Self::Relative)
	}
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let alt = f.alternate();

		match self {
			Self::Relative(offset) => {
				if alt {
					f.write_char('[')?;
				}

				Display::fmt(&offset, f)?;

				if alt {
					f.write_char(']')?;
				}
			}
		}

		Ok(())
	}
}

impl Div for Offset {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Div::div(lhs, rhs)),
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(Div::div(lhs, rhs)),
		}
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
		Self::Relative(value)
	}
}

impl From<&isize> for Offset {
	fn from(value: &isize) -> Self {
		(*value).into()
	}
}

impl Mul for Offset {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Mul::mul(lhs, rhs)),
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(Mul::mul(lhs, rhs)),
		}
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
		match self {
			Self::Relative(v) => Self::Relative(Neg::neg(v)),
		}
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
		match self {
			Self::Relative(v) => Self::Relative(Not::not(v)),
		}
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
		match self {
			Self::Relative(lhs) => lhs.eq(other),
		}
	}
}

impl PartialOrd<isize> for Offset {
	fn partial_cmp(&self, other: &isize) -> Option<Ordering> {
		let Self::Relative(offset) = *self;

		Some(offset.cmp(other))
	}
}

impl Rem for Offset {
	type Output = Self;

	fn rem(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Rem::rem(lhs, rhs)),
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(Rem::rem(lhs, rhs)),
		}
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
		match self {
			Self::Relative(value) => serializer.serialize_i64(*value as i64),
		}
	}
}

impl Sub for Offset {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => Self::Relative(Sub::sub(lhs, rhs)),
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(Sub::sub(lhs, rhs)),
		}
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
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => {
				Self::Relative(WrappingAdd::wrapping_add(lhs, rhs))
			}
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(WrappingAdd::wrapping_add(lhs, rhs)),
		}
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
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => {
				Self::Relative(WrappingDiv::wrapping_div(lhs, rhs))
			}
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(WrappingDiv::wrapping_div(lhs, rhs)),
		}
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
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => {
				Self::Relative(WrappingMul::wrapping_mul(lhs, rhs))
			}
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(WrappingMul::wrapping_mul(lhs, rhs)),
		}
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
		match self {
			Self::Relative(v) => Self::Relative(WrappingNeg::wrapping_neg(v)),
		}
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
		match (self, rhs) {
			(Self::Relative(lhs), Self::Relative(rhs)) => {
				Self::Relative(WrappingSub::wrapping_sub(lhs, rhs))
			}
		}
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
		match self {
			Self::Relative(lhs) => Self::Relative(WrappingSub::wrapping_sub(lhs, rhs)),
		}
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
