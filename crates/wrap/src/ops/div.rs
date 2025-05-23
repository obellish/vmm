pub trait WrappingDiv<Rhs = Self> {
	type Output;

	fn wrapping_div(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingDivAssign<Rhs = Self> {
	fn wrapping_div_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_div {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::WrappingDiv for $ty {
                type Output = Self;

                fn wrapping_div(self, rhs: Self) -> Self {
                    self.wrapping_div(rhs)
                }
            }

            impl $crate::ops::WrappingDivAssign for $ty {
                fn wrapping_div_assign(&mut self, rhs: Self) {
                    *self = self.wrapping_div(rhs);
                }
            }
        )*
    };
}

impl_wrapping_div!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
