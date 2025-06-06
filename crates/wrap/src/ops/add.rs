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
	($left: ty, $right: ty, $func: ident) => {
		impl $crate::ops::WrappingAdd<$right> for $left {
			type Output = $left;

			fn wrapping_add(self, rhs: $right) -> Self {
				<$left>::$func(self, rhs)
			}
		}

		impl $crate::ops::WrappingAdd<$right> for &$left {
			type Output = $left;

			fn wrapping_add(self, rhs: $right) -> $left {
				<$left>::$func(*self, rhs)
			}
		}

		impl $crate::ops::WrappingAdd<&$right> for $left {
			type Output = $left;

			fn wrapping_add(self, rhs: &$right) -> Self {
				<$left>::$func(self, *rhs)
			}
		}

		impl $crate::ops::WrappingAdd<&$right> for &$left {
			type Output = $left;

			fn wrapping_add(self, rhs: &$right) -> $left {
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
