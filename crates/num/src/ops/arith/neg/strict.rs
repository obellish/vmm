pub trait StrictNeg {
	type Output;

	fn strict_neg(self) -> Self::Output;
}

macro_rules! impl_strict_neg {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::StrictNeg for $ty {
                type Output = Self;

                fn strict_neg(self) -> Self {
                    <$ty>::strict_neg(self)
                }
            }

            impl $crate::ops::StrictNeg for &$ty {
                type Output = <$ty as $crate::ops::StrictNeg>::Output;

                fn strict_neg(self) -> Self::Output {
                    <$ty>::strict_neg(*self)
                }
            }
        )*
    };
}

impl_strict_neg!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
