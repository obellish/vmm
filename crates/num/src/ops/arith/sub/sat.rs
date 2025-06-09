pub trait SaturatingSub<Rhs = Self> {
    type Output;

    fn saturating_sub(self, rhs: Rhs) -> Self::Output;
}

pub trait SaturatingSubAssign<Rhs = Self> {
    fn saturating_sub_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_saturating_sub {
    ($signed:ty, $unsigned:ty) => {
        impl_saturating_sub!($signed, $signed, saturating_sub);
        impl_saturating_sub!($unsigned, $unsigned, saturating_sub);
        impl_saturating_sub!($signed, $unsigned, saturating_sub_unsigned);
        impl_saturating_sub!(@nightly $unsigned, $signed, saturating_sub_signed);
    };
    ($left:ty, $right:ty, $func:ident) => {
        impl $crate::ops::SaturatingSub<$right> for $left {
            type Output = Self;

            fn saturating_sub(self, rhs: $right) -> Self {
                <$left>::$func(self, rhs)
            }
        }

        impl $crate::ops::SaturatingSub<&$right> for $left {
            type Output = Self;

            fn saturating_sub(self, rhs: &$right) -> Self {
                <$left>::$func(self, *rhs)
            }
        }

        impl $crate::ops::SaturatingSub<$right> for &$left {
            type Output = <$left as $crate::ops::SaturatingSub<$right>>::Output;

            fn saturating_sub(self, rhs: $right) -> Self::Output {
                <$left>::$func(*self, rhs)
            }
        }

        impl $crate::ops::SaturatingSub<&$right> for &$left {
            type Output = <$left as $crate::ops::SaturatingSub<$right>>::Output;

            fn saturating_sub(self, rhs: &$right) -> Self::Output {
                <$left>::$func(*self, *rhs)
            }
        }

        impl $crate::ops::SaturatingSubAssign<$right> for $left {
            fn saturating_sub_assign(&mut self, rhs: $right) {
                *self = <$left>::$func(*self, rhs);
            }
        }

        impl $crate::ops::SaturatingSubAssign<&$right> for $left {
            fn saturating_sub_assign(&mut self, rhs: &$right) {
                *self = <$left>::$func(*self, *rhs);
            }
        }
    };
    (@nightly $left:ty, $right:ty, $func:ident) => {
        #[cfg(feature = "nightly")]
        impl $crate::ops::SaturatingSub<$right> for $left {
            type Output = Self;

            fn saturating_sub(self, rhs: $right) -> Self {
                <$left>::$func(self, rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::SaturatingSub<&$right> for $left {
            type Output = Self;

            fn saturating_sub(self, rhs: &$right) -> Self {
                <$left>::$func(self, *rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::SaturatingSub<$right> for &$left {
            type Output = <$left as $crate::ops::SaturatingSub<$right>>::Output;

            fn saturating_sub(self, rhs: $right) -> Self::Output {
                <$left>::$func(*self, rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::SaturatingSub<&$right> for &$left {
            type Output = <$left as $crate::ops::SaturatingSub<$right>>::Output;

            fn saturating_sub(self, rhs: &$right) -> Self::Output {
                <$left>::$func(*self, *rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::SaturatingSubAssign<$right> for $left {
            fn saturating_sub_assign(&mut self, rhs: $right) {
                *self = <$left>::$func(*self, rhs);
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::SaturatingSubAssign<&$right> for $left {
            fn saturating_sub_assign(&mut self, rhs: &$right) {
                *self = <$left>::$func(*self, *rhs);
            }
        }
    };
}

impl_saturating_sub!(i8, u8);
impl_saturating_sub!(i16, u16);
impl_saturating_sub!(i32, u32);
impl_saturating_sub!(i64, u64);
impl_saturating_sub!(i128, u128);
impl_saturating_sub!(isize, usize);
