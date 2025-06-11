pub trait SaturatingDiv<Rhs = Self> {
	type Output;

	fn saturating_div(self, rhs: Rhs) -> Self::Output;
}

pub trait SaturatingDivAssign<Rhs = Self> {
	fn saturating_div_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_saturating_div {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::SaturatingDiv for $ty {
                type Output = Self;

                #[inline]
                fn saturating_div(self, rhs: Self) -> Self {
                    <$ty>::saturating_div(self, rhs)
                }
            }

            impl $crate::ops::SaturatingDiv<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn saturating_div(self, rhs: &Self) -> Self {
                    <$ty>::saturating_div(self, *rhs)
                }
            }

            impl $crate::ops::SaturatingDiv<$ty> for &$ty {
                type Output = <$ty as $crate::ops::SaturatingDiv>::Output;

                #[inline]
                fn saturating_div(self, rhs: $ty) -> Self::Output {
                    <$ty>::saturating_div(*self, rhs)
                }
            }

            impl $crate::ops::SaturatingDiv for &$ty {
                type Output = <$ty as $crate::ops::SaturatingDiv>::Output;

                #[inline]
                fn saturating_div(self, rhs: Self) -> Self::Output {
                    <$ty>::saturating_div(*self, *rhs)
                }
            }

            impl $crate::ops::SaturatingDivAssign for $ty {
                #[inline]
                fn saturating_div_assign(&mut self, rhs: Self) {
                    *self = <$ty>::saturating_div(*self, rhs);
                }
            }

            impl $crate::ops::SaturatingDivAssign<&Self> for $ty {
                #[inline]
                fn saturating_div_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::saturating_div(*self, *rhs);
                }
            }
        )*
    };
}

impl_saturating_div!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
