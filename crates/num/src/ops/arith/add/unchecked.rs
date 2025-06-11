pub unsafe trait UncheckedAdd<Rhs = Self> {
	type Output;

	unsafe fn unchecked_add(self, rhs: Rhs) -> Self::Output;
}

pub unsafe trait UncheckedAddAssign<Rhs = Self> {
	unsafe fn unchecked_add_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_unchecked_add {
    ($($ty:ty)*) => {
        $(
            unsafe impl $crate::ops::UncheckedAdd for $ty {
                type Output = Self;

                #[inline]
                unsafe fn unchecked_add(self, rhs: Self) -> Self {
                    unsafe { <$ty>::unchecked_add(self, rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedAdd<&Self> for $ty {
                type Output = Self;

                #[inline]
                unsafe fn unchecked_add(self, rhs: &Self) -> Self {
                    unsafe { <$ty>::unchecked_add(self, *rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedAdd<$ty> for &$ty {
                type Output = <$ty as $crate::ops::UncheckedAdd>::Output;

                #[inline]
                unsafe fn unchecked_add(self, rhs: $ty) -> Self::Output {
                    unsafe { <$ty>::unchecked_add(*self, rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedAdd for &$ty {
                type Output = <$ty as $crate::ops::UncheckedAdd>::Output;

                #[inline]
                unsafe fn unchecked_add(self, rhs: Self) -> Self::Output {
                    unsafe { <$ty>::unchecked_add(*self, *rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedAddAssign for $ty {
                #[inline]
                unsafe fn unchecked_add_assign(&mut self, rhs: Self) {
                    unsafe { *self = <$ty>::unchecked_add(*self, rhs); }
                }
            }

            unsafe impl $crate::ops::UncheckedAddAssign<&Self> for $ty {
                #[inline]
                unsafe fn unchecked_add_assign(&mut self, rhs: &Self) {
                    unsafe { *self = <$ty>::unchecked_add(*self, *rhs); }
                }
            }
        )*
    };
}

impl_unchecked_add!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
