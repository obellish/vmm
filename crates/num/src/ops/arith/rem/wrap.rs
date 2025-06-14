pub trait WrappingRem<Rhs = Self> {
	type Output;

	fn wrapping_rem(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingRemAssign<Rhs = Self> {
	fn wrapping_rem_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_rem {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::WrappingRem for $ty {
                type Output = Self;

                #[inline]
                fn wrapping_rem(self, rhs: Self) -> Self {
                    <$ty>::wrapping_rem(self, rhs)
                }
            }

            impl $crate::ops::WrappingRem<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn wrapping_rem(self, rhs: &Self) -> Self {
                    <$ty>::wrapping_rem(self, *rhs)
                }
            }

            impl $crate::ops::WrappingRem<$ty> for &$ty {
                type Output = <$ty as $crate::ops::WrappingRem>::Output;

                #[inline]
                fn wrapping_rem(self, rhs: $ty) -> Self::Output {
                    <$ty>::wrapping_rem(*self, rhs)
                }
            }

            impl $crate::ops::WrappingRem for &$ty {
                type Output = <$ty as $crate::ops::WrappingRem>::Output;

                #[inline]
                fn wrapping_rem(self, rhs: Self) -> Self::Output {
                    <$ty>::wrapping_rem(*self, *rhs)
                }
            }

            impl $crate::ops::WrappingRemAssign for $ty {
                #[inline]
                fn wrapping_rem_assign(&mut self, rhs: Self) {
                    *self = <$ty>::wrapping_rem(*self, rhs);
                }
            }

            impl $crate::ops::WrappingRemAssign<&Self> for $ty {
                #[inline]
                fn wrapping_rem_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::wrapping_rem(*self, *rhs);
                }
            }
        )*
    };
}

impl_wrapping_rem!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
