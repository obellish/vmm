pub trait StrictRem<Rhs = Self> {
	type Output;

	fn strict_rem(self, rhs: Rhs) -> Self::Output;
}

pub trait StrictRemAssign<Rhs = Self> {
	fn strict_rem_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_strict_rem {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::StrictRem for $ty {
                type Output = Self;

                #[inline]
                fn strict_rem(self, rhs: Self) -> Self {
                    <$ty>::strict_rem(self, rhs)
                }
            }

            impl $crate::ops::StrictRem<&Self> for $ty {
                type Output = Self;

                #[inline]
                fn strict_rem(self, rhs: &Self) -> Self {
                    <$ty>::strict_rem(self, *rhs)
                }
            }

            impl $crate::ops::StrictRem<$ty> for &$ty {
                type Output = $ty;

                #[inline]
                fn strict_rem(self, rhs: $ty) -> $ty {
                    <$ty>::strict_rem(*self, rhs)
                }
            }

            impl $crate::ops::StrictRem for &$ty {
                type Output = $ty;

                #[inline]
                fn strict_rem(self, rhs: Self) -> $ty {
                    <$ty>::strict_rem(*self, *rhs)
                }
            }

            impl $crate::ops::StrictRemAssign for $ty {
                #[inline]
                fn strict_rem_assign(&mut self, rhs: Self) {
                    *self = <$ty>::strict_rem(*self, rhs);
                }
            }

            impl $crate::ops::StrictRemAssign<&Self> for $ty {
                #[inline]
                fn strict_rem_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::strict_rem(*self, *rhs);
                }
            }
        )*
    };
}

impl_strict_rem!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
