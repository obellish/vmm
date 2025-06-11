pub trait CheckedRem<Rhs = Self> {
	type Output;

	fn checked_rem(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedRemAssign<Rhs = Self> {
	fn checked_rem_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_rem {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::CheckedRem for $ty {
                type Output = Self;

                #[inline]
                fn checked_rem(self, rhs: Self) -> ::core::option::Option<Self> {
                    <$ty>::checked_rem(self, rhs)
                }
            }

            impl $crate::ops::CheckedRem<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn checked_rem(self, rhs: &Self) -> ::core::option::Option<Self> {
                    <$ty>::checked_rem(self, *rhs)
                }
            }

            impl $crate::ops::CheckedRem<$ty> for &$ty {
                type Output = <$ty as $crate::ops::CheckedRem>::Output;

                #[inline]
                fn checked_rem(self, rhs: $ty) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_rem(*self, rhs)
                }
            }

            impl $crate::ops::CheckedRem for &$ty {
                type Output = <$ty as $crate::ops::CheckedRem>::Output;

                #[inline]
                fn checked_rem(self, rhs: Self) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_rem(*self, *rhs)
                }
            }

            impl $crate::ops::CheckedRemAssign for $ty {
                #[inline]
                fn checked_rem_assign(&mut self, rhs: Self) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_rem(*self, rhs) {
                        *self = value;
                    }
                }
            }

            impl $crate::ops::CheckedRemAssign<&Self> for $ty {
                #[inline]
                fn checked_rem_assign(&mut self, rhs: &Self) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_rem(*self, *rhs) {
                        *self = value;
                    }
                }
            }
        )*
    };
}

impl_checked_rem!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
