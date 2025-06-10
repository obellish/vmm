pub trait CheckedShl<Rhs = Self> {
	type Output;

	fn checked_shl(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedShlAssign<Rhs = Self> {
	fn checked_shl_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_checked_shl {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::CheckedShl<u32> for $ty {
                type Output = Self;

                fn checked_shl(self, rhs: u32) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_shl(self, rhs)
                }
            }

            impl $crate::ops::CheckedShl<&u32> for $ty {
                type Output = Self;

                fn checked_shl(self, rhs: &u32) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_shl(self, *rhs)
                }
            }

            impl $crate::ops::CheckedShl<u32> for &$ty {
                type Output = <$ty as $crate::ops::CheckedShl<u32>>::Output;

                fn checked_shl(self, rhs: u32) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_shl(*self, rhs)
                }
            }

            impl $crate::ops::CheckedShl<&u32> for &$ty {
                type Output = <$ty as $crate::ops::CheckedShl<u32>>::Output;

                fn checked_shl(self, rhs: &u32) -> ::core::option::Option<Self::Output> {
                    <$ty>::checked_shl(*self, *rhs)
                }
            }

            impl $crate::ops::CheckedShlAssign<u32> for $ty {
                fn checked_shl_assign(&mut self, rhs: u32) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_shl(*self, rhs) {
                        *self = value;
                    }
                }
            }

            impl $crate::ops::CheckedShlAssign<&u32> for $ty {
                fn checked_shl_assign(&mut self, rhs: &u32) {
                    if let ::core::option::Option::Some(value) = <$ty>::checked_shl(*self, *rhs) {
                        *self = value;
                    }
                }
            }
        )*
    };
}

impl_checked_shl!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
