use core::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign,
};

use vmm_num::ops::WrappingAdd;

use super::Offset;

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
		AddAssign::add_assign(self, *rhs);
	}
}

impl AddAssign<isize> for Offset {
	fn add_assign(&mut self, rhs: isize) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&isize> for Offset {
	fn add_assign(&mut self, rhs: &isize) {
		AddAssign::add_assign(self, *rhs);
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
		DivAssign::div_assign(self, *rhs);
	}
}

impl DivAssign<isize> for Offset {
	fn div_assign(&mut self, rhs: isize) {
		*self = Div::div(*self, rhs);
	}
}

impl DivAssign<&isize> for Offset {
	fn div_assign(&mut self, rhs: &isize) {
		DivAssign::div_assign(self, *rhs);
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
		MulAssign::mul_assign(self, *rhs);
	}
}

impl MulAssign<isize> for Offset {
	fn mul_assign(&mut self, rhs: isize) {
		*self = Mul::mul(*self, rhs);
	}
}

impl MulAssign<&isize> for Offset {
	fn mul_assign(&mut self, rhs: &isize) {
		MulAssign::mul_assign(self, *rhs);
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
		RemAssign::rem_assign(self, *rhs);
	}
}

impl RemAssign<isize> for Offset {
	fn rem_assign(&mut self, rhs: isize) {
		*self = Rem::rem(*self, rhs);
	}
}

impl RemAssign<&isize> for Offset {
	fn rem_assign(&mut self, rhs: &isize) {
		RemAssign::rem_assign(self, *rhs);
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
		SubAssign::sub_assign(self, *rhs);
	}
}

impl SubAssign<isize> for Offset {
	fn sub_assign(&mut self, rhs: isize) {
		*self = Sub::sub(*self, rhs);
	}
}

impl SubAssign<&isize> for Offset {
	fn sub_assign(&mut self, rhs: &isize) {
		SubAssign::sub_assign(self, *rhs);
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
