pub trait SaturatingAdd<Rhs = Self> {
	type Output;

	fn saturating_add(self, rhs: Rhs) -> Self::Output;
}

pub trait SaturatingAddAssign<Rhs = Self> {
	fn saturating_add_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_saturating_add {
	($signed:ty, $unsigned:ty) => {
		impl_saturating_add!($signed, $signed, saturating_add);
		impl_saturating_add!($unsigned, $unsigned, saturating_add);
		impl_saturating_add!($signed, $unsigned, saturating_add_unsigned);
		impl_saturating_add!($unsigned, $signed, saturating_add_signed);
	};
	($left:ty, $right:ty, $func:ident) => {
		impl $crate::ops::SaturatingAdd<$right> for $left {
			type Output = Self;

			fn saturating_add(self, rhs: $right) -> Self {
				<$left>::$func(self, rhs)
			}
		}

		impl $crate::ops::SaturatingAdd<&$right> for $left {
			type Output = Self;

			fn saturating_add(self, rhs: &$right) -> Self {
				<$left>::$func(self, *rhs)
			}
		}

		impl $crate::ops::SaturatingAdd<$right> for &$left {
			type Output = <$left as $crate::ops::SaturatingAdd<$right>>::Output;

			fn saturating_add(self, rhs: $right) -> Self::Output {
				<$left>::$func(*self, rhs)
			}
		}
	};
}

impl_saturating_add!(i8, u8);
impl_saturating_add!(i16, u16);
impl_saturating_add!(i32, u32);
impl_saturating_add!(i64, u64);
impl_saturating_add!(i128, u128);
impl_saturating_add!(isize, usize);
