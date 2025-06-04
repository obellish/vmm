pub trait WrappingNeg {
	type Output;

	fn wrapping_neg(self) -> Self::Output;
}

macro_rules! impl_wrapping_neg {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::WrappingNeg for $ty {
                type Output = Self;

                fn wrapping_neg(self) -> Self::Output {
                    <$ty>::wrapping_neg(self)
                }
            }

            impl $crate::ops::WrappingNeg for &$ty {
                type Output = $ty;

                fn wrapping_neg(self) -> Self::Output {
                    <$ty>::wrapping_neg(*self)
                }
            }
        )*
    };
}

impl_wrapping_neg!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
