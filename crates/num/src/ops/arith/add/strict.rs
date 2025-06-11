pub trait StrictAdd<Rhs = Self> {
	type Output;

	fn strict_add(self, rhs: Rhs) -> Self::Output;
}

pub trait StrictAddAssign<Rhs = Self> {
	fn strict_add_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_strict_add {
	($signed:ty, $unsigned:ty) => {
		impl_strict_add!($signed, $signed, strict_add);
		impl_strict_add!($unsigned, $unsigned, strict_add);
		impl_strict_add!($signed, $unsigned, strict_add_unsigned);
		impl_strict_add!($unsigned, $signed, strict_add_signed);
	};
	($left:ty, $right:ty, $func:ident) => {
		impl $crate::ops::StrictAdd<$right> for $left {
			type Output = Self;

			#[inline]
			fn strict_add(self, rhs: $right) -> Self {
				<$left>::$func(self, rhs)
			}
		}

		impl $crate::ops::StrictAdd<&$right> for $left {
			type Output = Self;

			#[inline]
			fn strict_add(self, rhs: &$right) -> Self {
				<$left>::$func(self, *rhs)
			}
		}

		impl $crate::ops::StrictAdd<$right> for &$left {
			type Output = <$left as $crate::ops::StrictAdd<$right>>::Output;

			#[inline]
			fn strict_add(self, rhs: $right) -> Self::Output {
				<$left>::$func(*self, rhs)
			}
		}

		impl $crate::ops::StrictAdd<&$right> for &$left {
			type Output = <$left as $crate::ops::StrictAdd<$right>>::Output;

			#[inline]
			fn strict_add(self, rhs: &$right) -> Self::Output {
				<$left>::$func(*self, *rhs)
			}
		}

		impl $crate::ops::StrictAddAssign<$right> for $left {
			#[inline]
			fn strict_add_assign(&mut self, rhs: $right) {
				*self = <$left>::$func(*self, rhs);
			}
		}

		impl $crate::ops::StrictAddAssign<&$right> for $left {
			#[inline]
			fn strict_add_assign(&mut self, rhs: &$right) {
				*self = <$left>::$func(*self, *rhs);
			}
		}
	};
}

impl_strict_add!(i8, u8);
impl_strict_add!(i16, u16);
impl_strict_add!(i32, u32);
impl_strict_add!(i64, u64);
impl_strict_add!(i128, u128);
impl_strict_add!(isize, usize);
