pub trait WrappingShr<Rhs = Self> {
	type Output;

	fn wrapping_shr(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingShrAssign<Rhs = Self> {
	fn wrapping_shr_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_shr {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::WrappingShr<u32> for $ty {
                type Output = Self;

                fn wrapping_shr(self, rhs: u32) -> Self {
                    <$ty>::wrapping_shr(self, rhs)
                }
            }

            impl $crate::ops::WrappingShr<&u32> for $ty {
                type Output = Self;

                fn wrapping_shr(self, rhs: &u32) -> Self {
                    <$ty>::wrapping_shr(self, *rhs)
                }
            }

            impl $crate::ops::WrappingShr<u32> for &$ty {
                type Output = <$ty as $crate::ops::WrappingShr<u32>>::Output;

                fn wrapping_shr(self, rhs: u32) -> Self::Output {
                    <$ty>::wrapping_shr(*self, rhs)
                }
            }

            impl $crate::ops::WrappingShr<&u32> for &$ty {
                type Output = <$ty as $crate::ops::WrappingShr<u32>>::Output;

                fn wrapping_shr(self, rhs: &u32) -> Self::Output {
                    <$ty>::wrapping_shr(*self, *rhs)
                }
            }

            impl $crate::ops::WrappingShrAssign<u32> for $ty {
                fn wrapping_shr_assign(&mut self, rhs: u32) {
                    *self = <$ty>::wrapping_shr(*self, rhs);
                }
            }

            impl $crate::ops::WrappingShrAssign<&u32> for $ty {
                fn wrapping_shr_assign(&mut self, rhs: &u32) {
                    *self = <$ty>::wrapping_shr(*self, *rhs);
                }
            }
        )*
    };
}

impl_wrapping_shr!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
