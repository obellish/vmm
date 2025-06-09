pub trait CheckedSub<Rhs = Self> {
	type Output;

	fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedSubAssign<Rhs = Self> {
	fn checked_sub_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_sub {
    ($signed:ty, $unsigned:ty) => {
        impl_checked_sub!($signed, $signed, checked_sub);
        impl_checked_sub!($unsigned, $unsigned, checked_sub);
        impl_checked_sub!($signed, $unsigned, checked_sub_unsigned);
        impl_checked_sub!(@nightly $unsigned, $signed, checked_sub_signed);
    };
    ($left:ty, $right:ty, $func:ident) => {
        impl $crate::ops::CheckedSub<$right> for $left {
            type Output = Self;

            fn checked_sub(self, rhs: $right) -> ::core::option::Option<Self> {
                <$left>::$func(self, rhs)
            }
        }

        impl $crate::ops::CheckedSub<&$right> for $left {
            type Output = Self;

            fn checked_sub(self, rhs: &$right) -> ::core::option::Option<Self> {
                <$left>::$func(self, *rhs)
            }
        }

        impl $crate::ops::CheckedSub<$right> for &$left {
            type Output = <$left as $crate::ops::CheckedSub<$right>>::Output;

            fn checked_sub(self, rhs: $right) -> ::core::option::Option<Self::Output> {
                <$left>::$func(*self, rhs)
            }
        }

        impl $crate::ops::CheckedSub<&$right> for &$left {
            type Output = <$left as $crate::ops::CheckedSub<$right>>::Output;

            fn checked_sub(self, rhs: &$right) -> ::core::option::Option<Self::Output> {
                <$left>::$func(*self, *rhs)
            }
        }

        impl $crate::ops::CheckedSubAssign<$right> for $left {
            fn checked_sub_assign(&mut self, rhs: $right) {
                if let ::core::option::Option::Some(value) = <$left>::$func(*self, rhs) {
                    *self = value;
                }
            }
        }

        impl $crate::ops::CheckedSubAssign<&$right> for $left {
            fn checked_sub_assign(&mut self, rhs: &$right) {
                if let ::core::option::Option::Some(value) = <$left>::$func(*self, *rhs) {
                    *self = value;
                }
            }
        }
    };
    (@nightly $left:ty, $right:ty, $func:ident) => {
        #[cfg(feature = "nightly")]
        impl $crate::ops::CheckedSub<$right> for $left {
            type Output = Self;

            fn checked_sub(self, rhs: $right) -> ::core::option::Option<Self> {
                <$left>::$func(self, rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::CheckedSub<&$right> for $left {
            type Output = Self;

            fn checked_sub(self, rhs: &$right) -> ::core::option::Option<Self> {
                <$left>::$func(self, *rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::CheckedSub<$right> for &$left {
            type Output = <$left as $crate::ops::CheckedSub<$right>>::Output;

            fn checked_sub(self, rhs: $right) -> ::core::option::Option<Self::Output> {
                <$left>::$func(*self, rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::CheckedSub<&$right> for &$left {
            type Output = <$left as $crate::ops::CheckedSub<$right>>::Output;

            fn checked_sub(self, rhs: &$right) -> ::core::option::Option<Self::Output> {
                <$left>::$func(*self, *rhs)
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::CheckedSubAssign<$right> for $left {
            fn checked_sub_assign(&mut self, rhs: $right) {
                if let ::core::option::Option::Some(value) = <$left>::$func(*self, rhs) {
                    *self = value;
                }
            }
        }

        #[cfg(feature = "nightly")]
        impl $crate::ops::CheckedSubAssign<&$right> for $left {
            fn checked_sub_assign(&mut self, rhs: &$right) {
                if let ::core::option::Option::Some(value) = <$left>::$func(*self, *rhs) {
                    *self = value;
                }
            }
        }
    };
}

impl_checked_sub!(i8, u8);
impl_checked_sub!(i16, u16);
impl_checked_sub!(i32, u32);
impl_checked_sub!(i64, u64);
impl_checked_sub!(i128, u128);
impl_checked_sub!(isize, usize);
