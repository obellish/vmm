use core::ops::*;

use vmm_num::ops::*;

use super::Cell;

impl Add for Cell {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self::create(Add::add(self.value(), rhs.value()), self.index)
	}
}

impl Add<&Self> for Cell {
	type Output = Self;

	fn add(self, rhs: &Self) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<Cell> for &Cell {
	type Output = Cell;

	fn add(self, rhs: Cell) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add for &Cell {
	type Output = Cell;

	fn add(self, rhs: Self) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl Add<u8> for Cell {
	type Output = Self;

	fn add(self, rhs: u8) -> Self::Output {
		Self::create(Add::add(self.value(), rhs), self.index)
	}
}

impl Add<&u8> for Cell {
	type Output = Self;

	fn add(self, rhs: &u8) -> Self::Output {
		Add::add(self, *rhs)
	}
}

impl Add<u8> for &Cell {
	type Output = Cell;

	fn add(self, rhs: u8) -> Self::Output {
		Add::add(*self, rhs)
	}
}

impl Add<&u8> for &Cell {
	type Output = Cell;

	fn add(self, rhs: &u8) -> Self::Output {
		Add::add(*self, *rhs)
	}
}

impl AddAssign for Cell {
	fn add_assign(&mut self, rhs: Self) {
		*self = Add::add(*self, rhs);
	}
}

impl AddAssign<&Self> for Cell {
	fn add_assign(&mut self, rhs: &Self) {
		AddAssign::add_assign(self, *rhs);
	}
}

impl WrappingAdd for Cell {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		Self::create(
			WrappingAdd::wrapping_add(self.value(), rhs.value()),
			self.index,
		)
	}
}

impl WrappingAdd<&Self> for Cell {
	type Output = Self;

	fn wrapping_add(self, rhs: &Self) -> Self::Output {
		WrappingAdd::wrapping_add(self, *rhs)
	}
}

impl WrappingAdd<Cell> for &Cell {
	type Output = Cell;

	fn wrapping_add(self, rhs: Cell) -> Self::Output {
		WrappingAdd::wrapping_add(*self, rhs)
	}
}

impl WrappingAdd for &Cell {
	type Output = Cell;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		WrappingAdd::wrapping_add(*self, *rhs)
	}
}

impl WrappingAdd<i8> for Cell {
	type Output = Self;

	fn wrapping_add(self, rhs: i8) -> Self::Output {
		Self::create(WrappingAdd::wrapping_add(self.value(), rhs), self.index)
	}
}

impl WrappingAdd<&i8> for Cell {
	type Output = Self;

	fn wrapping_add(self, rhs: &i8) -> Self::Output {
		WrappingAdd::wrapping_add(self, *rhs)
	}
}

impl WrappingAdd<i8> for &Cell {
	type Output = Cell;

	fn wrapping_add(self, rhs: i8) -> Self::Output {
		WrappingAdd::wrapping_add(*self, rhs)
	}
}

impl WrappingAdd<&i8> for &Cell {
	type Output = Cell;

	fn wrapping_add(self, rhs: &i8) -> Self::Output {
		WrappingAdd::wrapping_add(*self, *rhs)
	}
}

impl WrappingAdd<u8> for Cell {
	type Output = Self;

	fn wrapping_add(self, rhs: u8) -> Self::Output {
		Self::create(WrappingAdd::wrapping_add(self.value(), rhs), self.index)
	}
}

impl WrappingAdd<&u8> for Cell {
	type Output = Self;

	fn wrapping_add(self, rhs: &u8) -> Self::Output {
		WrappingAdd::wrapping_add(self, *rhs)
	}
}

impl WrappingAdd<u8> for &Cell {
	type Output = Cell;

	fn wrapping_add(self, rhs: u8) -> Self::Output {
		WrappingAdd::wrapping_add(*self, rhs)
	}
}

impl WrappingAdd<&u8> for &Cell {
	type Output = Cell;

	fn wrapping_add(self, rhs: &u8) -> Self::Output {
		WrappingAdd::wrapping_add(*self, *rhs)
	}
}

impl WrappingAddAssign for Cell {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = WrappingAdd::wrapping_add(*self, rhs);
	}
}

impl WrappingAddAssign<&Self> for Cell {
	fn wrapping_add_assign(&mut self, rhs: &Self) {
		WrappingAddAssign::wrapping_add_assign(self, *rhs);
	}
}

impl WrappingAddAssign<i8> for Cell {
	fn wrapping_add_assign(&mut self, rhs: i8) {
		*self = WrappingAdd::wrapping_add(*self, rhs);
	}
}

impl WrappingAddAssign<&i8> for Cell {
	fn wrapping_add_assign(&mut self, rhs: &i8) {
		WrappingAddAssign::wrapping_add_assign(self, *rhs);
	}
}

impl WrappingAddAssign<u8> for Cell {
	fn wrapping_add_assign(&mut self, rhs: u8) {
		*self = WrappingAdd::wrapping_add(*self, rhs);
	}
}

impl WrappingAddAssign<&u8> for Cell {
	fn wrapping_add_assign(&mut self, rhs: &u8) {
		WrappingAddAssign::wrapping_add_assign(self, *rhs);
	}
}

impl WrappingMul<u8> for Cell {
	type Output = Self;

	fn wrapping_mul(self, rhs: u8) -> Self::Output {
		Self::create(WrappingMul::wrapping_mul(self.value(), rhs), self.index)
	}
}

impl WrappingMulAssign<u8> for Cell {
	fn wrapping_mul_assign(&mut self, rhs: u8) {
		*self = WrappingMul::wrapping_mul(*self, rhs);
	}
}

impl WrappingSub for Cell {
	type Output = Self;

	fn wrapping_sub(self, rhs: Self) -> Self::Output {
		Self::create(
			WrappingSub::wrapping_sub(self.value(), rhs.value()),
			self.index,
		)
	}
}

impl WrappingSub<u8> for Cell {
	type Output = Self;

	fn wrapping_sub(self, rhs: u8) -> Self::Output {
		Self::create(WrappingSub::wrapping_sub(self.value(), rhs), self.index)
	}
}

impl WrappingSubAssign for Cell {
	fn wrapping_sub_assign(&mut self, rhs: Self) {
		*self = WrappingSub::wrapping_sub(*self, rhs);
	}
}

impl WrappingSubAssign<u8> for Cell {
	fn wrapping_sub_assign(&mut self, rhs: u8) {
		*self = WrappingSub::wrapping_sub(*self, rhs);
	}
}
