pub trait CheckedAdd<Rhs = Self> {
	type Output;

	fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedAddAssign<Rhs = Self> {
	fn checked_add_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_add {
	($signed:ty, $unsigned:ty) => {
		impl_checked_add!($signed, $signed, checked_add);
		impl_checked_add!($unsigned, $unsigned, checked_add);
		impl_checked_add!($signed, $unsigned, checked_add_unsigned);
		impl_checked_add!($unsigned, $signed, checked_add_signed);
	};
	($left:ty, $right:ty, $func:ident) => {
		impl $crate::ops::CheckedAdd<$right> for $left {
			type Output = Self;

			#[inline]
			fn checked_add(self, rhs: $right) -> ::core::option::Option<Self> {
				<$left>::$func(self, rhs)
			}
		}

		impl $crate::ops::CheckedAdd<&$right> for $left {
			type Output = Self;

			#[inline]
			fn checked_add(self, rhs: &$right) -> ::core::option::Option<Self> {
				<$left>::$func(self, *rhs)
			}
		}

		impl $crate::ops::CheckedAdd<$right> for &$left {
			type Output = <$left as $crate::ops::CheckedAdd<$right>>::Output;

			#[inline]
			fn checked_add(self, rhs: $right) -> ::core::option::Option<Self::Output> {
				<$left>::$func(*self, rhs)
			}
		}

		impl $crate::ops::CheckedAdd<&$right> for &$left {
			type Output = <$left as $crate::ops::CheckedAdd<$right>>::Output;

			#[inline]
			fn checked_add(self, rhs: &$right) -> ::core::option::Option<Self::Output> {
				<$left>::$func(*self, *rhs)
			}
		}

		impl $crate::ops::CheckedAddAssign<$right> for $left {
			#[inline]
			fn checked_add_assign(&mut self, rhs: $right) {
				if let ::core::option::Option::Some(value) = <$left>::$func(*self, rhs) {
					*self = value;
				}
			}
		}

		impl $crate::ops::CheckedAddAssign<&$right> for $left {
			#[inline]
			fn checked_add_assign(&mut self, rhs: &$right) {
				if let ::core::option::Option::Some(value) = <$left>::$func(*self, *rhs) {
					*self = value;
				}
			}
		}
	};
}

impl_checked_add!(i8, u8);
impl_checked_add!(i16, u16);
impl_checked_add!(i32, u32);
impl_checked_add!(i64, u64);
impl_checked_add!(i128, u128);
impl_checked_add!(isize, usize);
