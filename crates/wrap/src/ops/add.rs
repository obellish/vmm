pub trait WrappingAdd<Rhs = Self> {
	type Output;

	fn wrapping_add(self, rhs: Rhs) -> Self::Output;
}

impl WrappingAdd for i8 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd for u8 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd<u8> for i8 {
	type Output = Self;

	fn wrapping_add(self, rhs: u8) -> Self::Output {
		self.wrapping_add_unsigned(rhs)
	}
}

impl WrappingAdd<i8> for u8 {
	type Output = Self;

	fn wrapping_add(self, rhs: i8) -> Self::Output {
		self.wrapping_add_signed(rhs)
	}
}

impl WrappingAdd for i16 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd for u16 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd<u16> for i16 {
	type Output = Self;

	fn wrapping_add(self, rhs: u16) -> Self::Output {
		self.wrapping_add_unsigned(rhs)
	}
}

impl WrappingAdd<i16> for u16 {
	type Output = Self;

	fn wrapping_add(self, rhs: i16) -> Self::Output {
		self.wrapping_add_signed(rhs)
	}
}

impl WrappingAdd for i32 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd for u32 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd<u32> for i32 {
	type Output = Self;

	fn wrapping_add(self, rhs: u32) -> Self::Output {
		self.wrapping_add_unsigned(rhs)
	}
}

impl WrappingAdd<i32> for u32 {
	type Output = Self;

	fn wrapping_add(self, rhs: i32) -> Self::Output {
		self.wrapping_add_signed(rhs)
	}
}

impl WrappingAdd for i64 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd for u64 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd<u64> for i64 {
	type Output = Self;

	fn wrapping_add(self, rhs: u64) -> Self::Output {
		self.wrapping_add_unsigned(rhs)
	}
}

impl WrappingAdd<i64> for u64 {
	type Output = Self;

	fn wrapping_add(self, rhs: i64) -> Self::Output {
		self.wrapping_add_signed(rhs)
	}
}

impl WrappingAdd for i128 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd for u128 {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd<u128> for i128 {
	type Output = Self;

	fn wrapping_add(self, rhs: u128) -> Self::Output {
		self.wrapping_add_unsigned(rhs)
	}
}

impl WrappingAdd<i128> for u128 {
	type Output = Self;

	fn wrapping_add(self, rhs: i128) -> Self::Output {
		self.wrapping_add_signed(rhs)
	}
}

impl WrappingAdd for isize {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd for usize {
	type Output = Self;

	fn wrapping_add(self, rhs: Self) -> Self::Output {
		self.wrapping_add(rhs)
	}
}

impl WrappingAdd<usize> for isize {
	type Output = Self;

	fn wrapping_add(self, rhs: usize) -> Self::Output {
		self.wrapping_add_unsigned(rhs)
	}
}

impl WrappingAdd<isize> for usize {
	type Output = Self;

	fn wrapping_add(self, rhs: isize) -> Self::Output {
		self.wrapping_add_signed(rhs)
	}
}

pub trait WrappingAddAssign<Rhs = Self> {
	fn wrapping_add_assign(&mut self, rhs: Rhs);
}

impl WrappingAddAssign for i8 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<u8> for i8 {
	fn wrapping_add_assign(&mut self, rhs: u8) {
		*self = self.wrapping_add_unsigned(rhs);
	}
}

impl WrappingAddAssign for u8 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<i8> for u8 {
	fn wrapping_add_assign(&mut self, rhs: i8) {
		*self = self.wrapping_add_signed(rhs);
	}
}

impl WrappingAddAssign for i16 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<u16> for i16 {
	fn wrapping_add_assign(&mut self, rhs: u16) {
		*self = self.wrapping_add_unsigned(rhs);
	}
}

impl WrappingAddAssign for u16 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<i16> for u16 {
	fn wrapping_add_assign(&mut self, rhs: i16) {
		*self = self.wrapping_add_signed(rhs);
	}
}

impl WrappingAddAssign for i32 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<u32> for i32 {
	fn wrapping_add_assign(&mut self, rhs: u32) {
		*self = self.wrapping_add_unsigned(rhs);
	}
}

impl WrappingAddAssign for u32 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<i32> for u32 {
	fn wrapping_add_assign(&mut self, rhs: i32) {
		*self = self.wrapping_add_signed(rhs);
	}
}

impl WrappingAddAssign for i64 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<u64> for i64 {
	fn wrapping_add_assign(&mut self, rhs: u64) {
		*self = self.wrapping_add_unsigned(rhs);
	}
}

impl WrappingAddAssign for u64 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<i64> for u64 {
	fn wrapping_add_assign(&mut self, rhs: i64) {
		*self = self.wrapping_add_signed(rhs);
	}
}

impl WrappingAddAssign for i128 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<u128> for i128 {
	fn wrapping_add_assign(&mut self, rhs: u128) {
		*self = self.wrapping_add_unsigned(rhs);
	}
}

impl WrappingAddAssign for u128 {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<i128> for u128 {
	fn wrapping_add_assign(&mut self, rhs: i128) {
		*self = self.wrapping_add_signed(rhs);
	}
}

impl WrappingAddAssign for isize {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<usize> for isize {
	fn wrapping_add_assign(&mut self, rhs: usize) {
		*self = self.wrapping_add_unsigned(rhs);
	}
}

impl WrappingAddAssign for usize {
	fn wrapping_add_assign(&mut self, rhs: Self) {
		*self = self.wrapping_add(rhs);
	}
}

impl WrappingAddAssign<isize> for usize {
	fn wrapping_add_assign(&mut self, rhs: isize) {
		*self = self.wrapping_add_signed(rhs);
	}
}

#[cfg(test)]
mod tests {
	use core::{fmt::Debug, ops::Add};

	use crate::{Wrapping, ops::WrappingAdd};

	#[test]
	fn additions() {
		additions_inner(10i8, 8, 10u8, 9, i8::MAX, u8::MAX, i8::MIN, 1, -1);
		additions_inner(10i16, 8, 10u16, 9, i16::MAX, u16::MAX, i16::MIN, 1, -1);
		additions_inner(10i32, 8, 10u32, 9, i32::MAX, u32::MAX, i32::MIN, 1, -1);
		additions_inner(10i64, 8, 10u64, 9, i64::MAX, u64::MAX, i64::MIN, 1, -1);
		additions_inner(10i128, 8, 10u128, 9, i128::MAX, u128::MAX, i128::MIN, 1, -1);
		additions_inner(
			10isize,
			8,
			10usize,
			9,
			isize::MAX,
			usize::MAX,
			isize::MIN,
			1,
			-1,
		);
	}

	// Overly complex, however the alternative is macros
	fn additions_inner<Signed, Unsigned>(
		signed_ten: Signed,
		signed_eight: Signed,
		unsigned_ten: Unsigned,
		unsigned_nine: Unsigned,
		signed_max: Signed,
		unsigned_max: Unsigned,
		signed_min: Signed,
		unsigned_one: Unsigned,
		negative_one: Signed,
	) where
		Signed: Copy
			+ Debug
			+ Default
			+ Eq
			+ WrappingAdd<Output = Signed>
			+ WrappingAdd<Unsigned, Output = Signed>,
		Unsigned: Add<Output = Unsigned>
			+ Copy
			+ Debug
			+ Default
			+ Eq
			+ WrappingAdd<Output = Unsigned>
			+ WrappingAdd<Signed, Output = Unsigned>,
	{
		{
			let value = Wrapping(signed_ten);

			let result = value + signed_max;

			let result = result + signed_max;

			assert_eq!(result.0, signed_eight);
		}

		{
			let value: Wrapping<Signed> = Wrapping::default();

			let result = value + unsigned_max;

			assert_eq!(result.0, negative_one);
		}

		{
			let value = Wrapping(unsigned_ten);

			let result = value + unsigned_max;

			assert_eq!(result.0, unsigned_nine);
		}

		{
			let value: Wrapping<Unsigned> = Wrapping::default();

			let result = value + signed_min;

			assert_eq!(
				result.0,
				unsafe { core::mem::transmute_copy::<Signed, Unsigned>(&signed_max) }
					+ unsigned_one
			);
		}
	}
}
