pub trait CheckedMul<Rhs = Self> {
	type Output;

	fn checked_mul(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedMulAssign<Rhs = Self> {
	fn checked_mul_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_mul {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::CheckedMul for $ty {
                type Output = Self;

                #[inline]
                fn checked_mul(self, rhs: Self) -> ::core::option::Option<Self> {
                    <$ty>::checked_mul(self, rhs)
                }
            }

            impl $crate::ops::CheckedMul<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn checked_mul(self, rhs: &Self) -> ::core::option::Option<Self> {
                    <$ty>::checked_mul(self, *rhs)
                }
            }

            impl $crate::ops::CheckedMul<$ty> for &$ty {
                type Output = <$ty as $crate::ops::CheckedMul>::Output;

                #[inline]
                fn checked_mul(self, rhs: $ty) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_mul(*self, rhs)
                }
            }

            impl $crate::ops::CheckedMul for &$ty {
                type Output = <$ty as $crate::ops::CheckedMul>::Output;

                #[inline]
                fn checked_mul(self, rhs: Self) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_mul(*self, *rhs)
                }
            }

            impl $crate::ops::CheckedMulAssign for $ty {
                #[inline]
                fn checked_mul_assign(&mut self, rhs: Self) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_mul(*self, rhs) {
                        *self = value;
                    }
                }
            }

            impl $crate::ops::CheckedMulAssign<&Self> for $ty {
                #[inline]
                fn checked_mul_assign(&mut self, rhs: &Self) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_mul(*self, *rhs) {
                        *self = value;
                    }
                }
            }
        )*
    };
}

impl_checked_mul!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
