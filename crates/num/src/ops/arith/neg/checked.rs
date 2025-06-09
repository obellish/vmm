pub trait CheckedNeg {
	type Output;

	fn checked_neg(self) -> Option<Self::Output>;
}

macro_rules! impl_checked_neg {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::CheckedNeg for $ty {
                type Output = Self;

                fn checked_neg(self) -> ::core::option::Option<Self> {
                    <$ty>::checked_neg(self)
                }
            }

            impl $crate::ops::CheckedNeg for &$ty {
                type Output = <$ty as $crate::ops::CheckedNeg>::Output;

                fn checked_neg(self) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_neg(*self)
                }
            }
        )*
    };
}

impl_checked_neg!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
