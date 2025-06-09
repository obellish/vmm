pub trait SaturatingMul<Rhs = Self> {
    type Output;

    fn saturating_mul(self, rhs: Rhs) -> Self::Output;
}

pub trait SaturatingMulAssign<Rhs = Self> {
    fn saturating_mul_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_saturating_mul {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::SaturatingMul for $ty {
                type Output = Self;

                fn saturating_mul(self, rhs: Self) -> Self {
                    <$ty>::saturating_mul(self, rhs)
                }
            }

            impl $crate::ops::SaturatingMul<&Self> for $ty {
                type Output = Self;

                fn saturating_mul(self, rhs: &Self) -> Self {
                    <$ty>::saturating_mul(self, *rhs)
                }
            }

            impl $crate::ops::SaturatingMul<$ty> for &$ty {
                type Output = <$ty as $crate::ops::SaturatingMul>::Output;

                fn saturating_mul(self, rhs: $ty) -> Self::Output {
                    <$ty>::saturating_mul(*self, rhs)
                }
            }

            impl $crate::ops::SaturatingMul for &$ty {
                type Output = <$ty as $crate::ops::SaturatingMul>::Output;

                fn saturating_mul(self, rhs: Self) -> Self::Output {
                    <$ty>::saturating_mul(*self, *rhs)
                }
            }

            impl $crate::ops::SaturatingMulAssign for $ty {
                fn saturating_mul_assign(&mut self, rhs: Self) {
                    *self = <$ty>::saturating_mul(*self, rhs);
                }
            }

            impl $crate::ops::SaturatingMulAssign<&Self> for $ty {
                fn saturating_mul_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::saturating_mul(*self, *rhs);
                }
            }
        )*
    };
}

impl_saturating_mul!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
