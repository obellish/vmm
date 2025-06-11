pub unsafe trait UncheckedMul<Rhs = Self> {
	type Output;

	unsafe fn unchecked_mul(self, rhs: Rhs) -> Self::Output;
}

pub unsafe trait UncheckedMulAssign<Rhs = Self> {
	unsafe fn unchecked_mul_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_unchecked_mul {
    ($($ty:ty)*) => {
        $(
            unsafe impl $crate::ops::UncheckedMul for $ty {
                type Output = Self;

                #[inline]
                unsafe fn unchecked_mul(self, rhs: Self) -> Self {
                    unsafe { <$ty>::unchecked_mul(self, rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedMul<&Self> for $ty {
                type Output = Self;

                #[inline]
                unsafe fn unchecked_mul(self, rhs: &Self) -> Self {
                    unsafe { <$ty>::unchecked_mul(self, *rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedMul<$ty> for &$ty {
                type Output = <$ty as $crate::ops::UncheckedMul>::Output;

                #[inline]
                unsafe fn unchecked_mul(self, rhs: $ty) -> Self::Output {
                    unsafe { <$ty>::unchecked_mul(*self, rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedMul for &$ty {
                type Output = <$ty as $crate::ops::UncheckedMul>::Output;

                #[inline]
                unsafe fn unchecked_mul(self, rhs: Self) -> Self::Output {
                    unsafe { <$ty>::unchecked_mul(*self, *rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedMulAssign for $ty {
                #[inline]
                unsafe fn unchecked_mul_assign(&mut self, rhs: Self) {
                    unsafe { *self = <$ty>::unchecked_mul(*self, rhs); }
                }
            }

            unsafe impl $crate::ops::UncheckedMulAssign<&Self> for $ty {
                #[inline]
                unsafe fn unchecked_mul_assign(&mut self, rhs: &Self) {
                    unsafe { *self = <$ty>::unchecked_mul(*self, *rhs); }
                }
            }
        )*
    };
}

impl_unchecked_mul!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
