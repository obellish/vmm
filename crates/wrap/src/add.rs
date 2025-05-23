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

#[cfg(test)]
mod tests {
	macro_rules! test_type {
		($signed:ty, $unsigned:ty) => {{
			let value: $crate::Wrapping<$signed> = $crate::Wrapping(10);

			let result = value + <$signed>::MAX;

			let result = result + <$signed>::MAX;

			assert_eq!(result.0, 8);
		}

		{
			let value: $crate::Wrapping<$signed> = $crate::Wrapping(0);

			let result = value + <$unsigned>::MAX;

			assert_eq!(result.0, -1);
		}

		{
			let value: $crate::Wrapping<$unsigned> = $crate::Wrapping(10);

			let result = value + <$unsigned>::MAX;

			assert_eq!(result.0, 9);
		}

		{
			let value: $crate::Wrapping<$unsigned> = $crate::Wrapping(0);

			let result = value + <$signed>::MIN;

			assert_eq!(result.0, <$signed>::MAX as $unsigned + 1);
		}};
	}

	#[test]
	fn additions() {
		{
			test_type!(i8, u8);
		}

		{
			test_type!(i16, u16);
		}

		{
			test_type!(i32, u32);
		}

		{
			test_type!(i64, u64);
		}

		{
			test_type!(i128, u128);
		}

		{
			test_type!(isize, usize);
		}
	}
}
