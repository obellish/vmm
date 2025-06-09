pub trait CheckedShr<Rhs = Self> {
	type Output;

	fn checked_shr(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedShrAssign<Rhs = Self> {
	fn checked_shr_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_shr {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::CheckedShr<u32> for $ty {
                type Output = Self;

                fn checked_shr(self, rhs: u32) -> ::core::option::Option<Self> {
                    <$ty>::checked_shr(self, rhs)
                }
            }

            impl $crate::ops::CheckedShr<&u32> for $ty {
                type Output = Self;

                fn checked_shr(self, rhs: &u32) -> ::core::option::Option<Self> {
                    <$ty>::checked_shr(self, *rhs)
                }
            }

            impl $crate::ops::CheckedShr<u32> for &$ty {
                type Output = <$ty as $crate::ops::CheckedShr<u32>>::Output;

                fn checked_shr(self, rhs: u32) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_shr(*self, rhs)
                }
            }

            impl $crate::ops::CheckedShr<&u32> for &$ty {
                type Output = <$ty as $crate::ops::CheckedShr<u32>>::Output;

                fn checked_shr(self, rhs: &u32) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_shr(*self, *rhs)
                }
            }

            impl $crate::ops::CheckedShrAssign<u32> for $ty {
                fn checked_shr_assign(&mut self, rhs: u32) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_shr(*self, rhs) {
                        *self = value;
                    }
                }
            }

            impl $crate::ops::CheckedShrAssign<&u32> for $ty {
                fn checked_shr_assign(&mut self, rhs: &u32) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_shr(*self, *rhs) {
                        *self = value;
                    }
                }
            }
        )*
    };
}

impl_checked_shr!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
