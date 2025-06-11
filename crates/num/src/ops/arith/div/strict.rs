pub trait StrictDiv<Rhs = Self> {
	type Output;

	fn strict_div(self, rhs: Rhs) -> Self::Output;
}

pub trait StrictDivAssign<Rhs = Self> {
	fn strict_div_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_strict_div {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::StrictDiv for $ty {
                type Output = Self;

                #[inline]
                fn strict_div(self, rhs: Self) -> Self {
                    <$ty>::strict_div(self, rhs)
                }
            }

            impl $crate::ops::StrictDiv<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn strict_div(self, rhs: &Self) -> Self {
                    <$ty>::strict_div(self, *rhs)
                }
            }

            impl $crate::ops::StrictDiv<$ty> for &$ty {
                type Output = <$ty as $crate::ops::StrictDiv>::Output;

                #[inline]
                fn strict_div(self, rhs: $ty) -> Self::Output {
                    <$ty>::strict_div(*self, rhs)
                }
            }

            impl $crate::ops::StrictDiv for &$ty {
                type Output = <$ty as $crate::ops::StrictDiv>::Output;

                #[inline]
                fn strict_div(self, rhs: Self) -> Self::Output {
                    <$ty>::strict_div(*self, *rhs)
                }
            }

            impl $crate::ops::StrictDivAssign for $ty {
                #[inline]
                fn strict_div_assign(&mut self, rhs: Self) {
                    *self = <$ty>::strict_div(*self, rhs);
                }
            }

            impl $crate::ops::StrictDivAssign<&Self> for $ty {
                #[inline]
                fn strict_div_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::strict_div(*self, *rhs);
                }
            }
        )*
    };
}

impl_strict_div!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
