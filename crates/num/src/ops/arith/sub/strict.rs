pub trait StrictSub<Rhs = Self> {
	type Output;

	fn strict_sub(self, rhs: Rhs) -> Self::Output;
}

pub trait StrictSubAssign<Rhs = Self> {
	fn strict_sub_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_strict_sub {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::StrictSub for $ty {
                type Output = Self;

                fn strict_sub(self, rhs: Self) -> Self::Output {
                    <$ty>::strict_sub(self, rhs)
                }
            }

            impl $crate::ops::StrictSub<&Self> for $ty {
                type Output = Self;

                fn strict_sub(self, rhs: &Self) -> Self {
                    <$ty>::strict_sub(self, *rhs)
                }
            }

            impl $crate::ops::StrictSub<$ty> for &$ty {
                type Output = $ty;

                fn strict_sub(self, rhs: $ty) -> Self::Output {
                    <$ty>::strict_sub(*self, rhs)
                }
            }

            impl $crate::ops::StrictSub for &$ty {
                type Output = $ty;

                fn strict_sub(self, rhs: Self) -> Self::Output {
                    <$ty>::strict_sub(*self, *rhs)
                }
            }

            impl $crate::ops::StrictSubAssign for $ty {
                fn strict_sub_assign(&mut self, rhs: Self) {
                    *self = <$ty>::strict_sub(*self, rhs);
                }
            }

            impl $crate::ops::StrictSubAssign<&Self> for $ty {
                fn strict_sub_assign(&mut self, rhs: &Self) {
                    *self = <$ty>::strict_sub(*self, *rhs);
                }
            }
        )*
    };
}

impl_strict_sub!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
