pub trait CheckedDiv<Rhs = Self> {
	type Output;

	fn checked_div(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedDivAssign<Rhs = Self> {
	fn checked_div_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_div {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::CheckedDiv for $ty {
                type Output = Self;

                fn checked_div(self, rhs: Self) -> ::core::option::Option<Self> {
                    <$ty>::checked_div(self, rhs)
                }
            }

            impl $crate::ops::CheckedDiv<&Self> for $ty {
                type Output = Self;

                fn checked_div(self, rhs: &Self) -> ::core::option::Option<Self> {
                    <$ty>::checked_div(self, *rhs)
                }
            }

            impl $crate::ops::CheckedDiv<$ty> for &$ty {
                type Output = <$ty as $crate::ops::CheckedDiv>::Output;

                fn checked_div(self, rhs: $ty) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_div(*self, rhs)
                }
            }

            impl $crate::ops::CheckedDiv for &$ty {
                type Output = <$ty as $crate::ops::CheckedDiv>::Output;

                fn checked_div(self, rhs: Self) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_div(*self, *rhs)
                }
            }

            impl $crate::ops::CheckedDivAssign for $ty {
                fn checked_div_assign(&mut self, rhs: Self) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_div(*self, rhs) {
                        *self = value;
                    }
                }
            }

            impl $crate::ops::CheckedDivAssign<&Self> for $ty {
                fn checked_div_assign(&mut self, rhs: &Self) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_div(*self, *rhs) {
                        *self = value;
                    }
                }
            }
        )*
    };
}

impl_checked_div!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
