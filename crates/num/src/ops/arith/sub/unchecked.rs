pub unsafe trait UncheckedSub<Rhs = Self> {
	type Output;

	unsafe fn unchecked_sub(self, rhs: Rhs) -> Self::Output;
}

pub unsafe trait UncheckedSubAssign<Rhs = Self> {
	unsafe fn unchecked_sub_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_unchecked_sub {
	($($ty:ty)*) => {
        $(
            unsafe impl $crate::ops::UncheckedSub for $ty {
                type Output = Self;

                unsafe fn unchecked_sub(self, rhs: Self) -> Self {
                    unsafe { <$ty>::unchecked_sub(self, rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedSub<&Self> for $ty {
                type Output = Self;

                unsafe fn unchecked_sub(self, rhs: &Self) -> Self {
                    unsafe { <$ty>::unchecked_sub(self, *rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedSub<$ty> for &$ty {
                type Output = <$ty as $crate::ops::UncheckedSub>::Output;

                unsafe fn unchecked_sub(self, rhs: $ty) -> Self::Output {
                    unsafe { <$ty>::unchecked_sub(*self, rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedSub for &$ty {
                type Output = <$ty as $crate::ops::UncheckedSub>::Output;

                unsafe fn unchecked_sub(self, rhs: Self) -> Self::Output {
                    unsafe { <$ty>::unchecked_sub(*self, *rhs) }
                }
            }

            unsafe impl $crate::ops::UncheckedSubAssign for $ty {
                unsafe fn unchecked_sub_assign(&mut self, rhs: Self) {
                    unsafe { *self = <$ty>::unchecked_sub(*self, rhs); }
                }
            }

            unsafe impl $crate::ops::UncheckedSubAssign<&Self> for $ty {
                unsafe fn unchecked_sub_assign(&mut self, rhs: &Self) {
                    unsafe { *self = <$ty>::unchecked_sub(*self, *rhs); }
                }
            }
        )*
    };
}

impl_unchecked_sub!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
