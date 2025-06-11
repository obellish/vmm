pub trait StrictMul<Rhs = Self> {
	type Output;

	fn strict_mul(self, rhs: Rhs) -> Self::Output;
}

pub trait StrictMulAssign<Rhs = Self> {
	fn strict_mul_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_strict_mul {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::StrictMul for $ty {
                type Output = Self;

                #[inline]
                fn strict_mul(self, rhs: Self) -> Self {
                    <$ty>::strict_mul(self, rhs)
                }
            }

            impl $crate::ops::StrictMul<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn strict_mul(self, rhs: &Self) -> Self {
                    <$ty>::strict_mul(self, *rhs)
                }
            }

            impl $crate::ops::StrictMul<$ty> for &$ty {
                type Output = <$ty as $crate::ops::StrictMul>::Output;

                #[inline]
                fn strict_mul(self, rhs: $ty) -> Self::Output {
                    <$ty>::strict_mul(*self, rhs)
                }
            }

            impl $crate::ops::StrictMul for &$ty {
                type Output = <$ty as $crate::ops::StrictMul>::Output;

                #[inline]
                fn strict_mul(self, rhs: Self) -> Self::Output {
                    <$ty>::strict_mul(*self, *rhs)
                }
            }

            impl $crate::ops::StrictMulAssign for $ty {
                #[inline]
                fn strict_mul_assign(&mut self, rhs: Self) {
                    *self = <$ty>::strict_mul(*self, rhs);
                }
            }

            impl $crate::ops::StrictMulAssign<&Self> for $ty {
                #[inline]
                fn strict_mul_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::strict_mul(*self, *rhs);
                }
            }
        )*
    };
}

impl_strict_mul!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
