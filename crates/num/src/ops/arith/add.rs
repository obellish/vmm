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
			type Output = <$left as $crate::ops::WrappingAdd>::Output;

			fn wrapping_add(self, rhs: $right) -> Self::Output {
				<$left>::$func(*self, rhs)
			}
		}

		impl $crate::ops::WrappingAdd<&$right> for &$left {
			type Output = <$left as $crate::ops::WrappingAdd>::Output;

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
