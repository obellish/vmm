pub trait WrappingAdd<Rhs = Self> {
	type Output;

	fn wrapping_add(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingAddAssign<Rhs = Self> {
	fn wrapping_add_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_add {
	($signed:ty, $unsigned:ty) => {
		impl_wrapping_add!($signed, $signed, wrapping_add);
		impl_wrapping_add!($unsigned, $unsigned, wrapping_add);
		impl_wrapping_add!($signed, $unsigned, wrapping_add_unsigned);
		impl_wrapping_add!($unsigned, $signed, wrapping_add_signed);
	};
	($left:ty, $right:ty, $func:ident) => {
		impl $crate::ops::WrappingAdd<$right> for $left {
			type Output = Self;

			fn wrapping_add(self, rhs: $right) -> Self {
				<$left>::$func(self, rhs)
			}
		}

		impl $crate::ops::WrappingAdd<&$right> for $left {
			type Output = Self;

			fn wrapping_add(self, rhs: &$right) -> Self {
				<$left>::$func(self, *rhs)
			}
		}

		impl $crate::ops::WrappingAdd<$right> for &$left {
			type Output = <$left as $crate::ops::WrappingAdd<$right>>::Output;

			fn wrapping_add(self, rhs: $right) -> Self::Output {
				<$left>::$func(*self, rhs)
			}
		}

		impl $crate::ops::WrappingAdd<&$right> for &$left {
			type Output = <$left as $crate::ops::WrappingAdd<$right>>::Output;

			fn wrapping_add(self, rhs: &$right) -> Self::Output {
				<$left>::$func(*self, *rhs)
			}
		}

		impl $crate::ops::WrappingAddAssign<$right> for $left {
			fn wrapping_add_assign(&mut self, rhs: $right) {
				*self = <$left>::$func(*self, rhs);
			}
		}

		impl $crate::ops::WrappingAddAssign<&$right> for $left {
			fn wrapping_add_assign(&mut self, rhs: &$right) {
				*self = <$left>::$func(*self, *rhs);
			}
		}
	};
}

impl_wrapping_add!(i8, u8);
impl_wrapping_add!(i16, u16);
impl_wrapping_add!(i32, u32);
impl_wrapping_add!(i64, u64);
impl_wrapping_add!(i128, u128);
impl_wrapping_add!(isize, usize);

#[cfg(test)]
mod tests {
	use crate::Wrapping;

	#[test]
	fn add() {
		assert_eq!(Wrapping::add(i8::MAX, 1i8), i8::MIN);
		assert_eq!(Wrapping::add(i16::MAX, 1i16), i16::MIN);
		assert_eq!(Wrapping::add(i32::MAX, 1i32), i32::MIN);
		assert_eq!(Wrapping::add(i64::MAX, 1i64), i64::MIN);
		assert_eq!(Wrapping::add(i128::MAX, 1i128), i128::MIN);
		assert_eq!(Wrapping::add(isize::MAX, 1isize), isize::MIN);

		assert_eq!(Wrapping::add(u8::MAX, 1u8), u8::MIN);
		assert_eq!(Wrapping::add(u16::MAX, 1u16), u16::MIN);
		assert_eq!(Wrapping::add(u32::MAX, 1u32), u32::MIN);
		assert_eq!(Wrapping::add(u64::MAX, 1u64), u64::MIN);
		assert_eq!(Wrapping::add(u128::MAX, 1u128), u128::MIN);
		assert_eq!(Wrapping::add(usize::MAX, 1usize), usize::MIN);
	}

	#[test]
	fn sub() {
		assert_eq!(Wrapping::sub(i8::MIN, 1i8), i8::MAX);
		assert_eq!(Wrapping::sub(i16::MIN, 1i16), i16::MAX);
		assert_eq!(Wrapping::sub(i32::MIN, 1i32), i32::MAX);
		assert_eq!(Wrapping::sub(i64::MIN, 1i64), i64::MAX);
		assert_eq!(Wrapping::sub(i128::MIN, 1i128), i128::MAX);
		assert_eq!(Wrapping::sub(isize::MIN, 1isize), isize::MAX);

		assert_eq!(Wrapping::sub(u8::MIN, 1u8), u8::MAX);
		assert_eq!(Wrapping::sub(u16::MIN, 1u16), u16::MAX);
		assert_eq!(Wrapping::sub(u32::MIN, 1u32), u32::MAX);
		assert_eq!(Wrapping::sub(u64::MIN, 1u64), u64::MAX);
		assert_eq!(Wrapping::sub(u128::MIN, 1u128), u128::MAX);
		assert_eq!(Wrapping::sub(usize::MIN, 1usize), usize::MAX);
	}

	#[test]
	fn mul() {
		assert_eq!(Wrapping::mul(0xfeu8 as i8, 16i8), 0xe0u8 as i8);
		assert_eq!(Wrapping::mul(0xfedcu16 as i16, 16i16), 0xedc0u16 as i16);
		assert_eq!(
			Wrapping::mul(0xfedc_ba98u32 as i32, 16i32),
			0xedcb_a980u32 as i32
		);
		assert_eq!(
			Wrapping::mul(0xfedc_ba98_7654_3217u64 as i64, 16i64),
			0xedcb_a987_6543_2170_u64 as i64
		);

		assert_eq!(Wrapping::mul(0xfeu8, 16), 0xe0);
		assert_eq!(Wrapping::mul(0xfedcu16, 16), 0xedc0);
		assert_eq!(Wrapping::mul(0xfedc_ba98_u32, 16), 0xedcb_a980);
		assert_eq!(
			Wrapping::mul(0xfedc_ba98_7654_3217_u64, 16),
			0xedcb_a987_6543_2170
		);
	}
}
