pub trait WrappingMul<Rhs = Self> {
	type Output;

	fn wrapping_mul(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingMulAssign<Rhs = Self> {
	fn wrapping_mul_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_mul {
    ($($ty:ty)*) => {
        $(
            impl $crate::ops::WrappingMul for $ty {
                type Output = Self;

                fn wrapping_mul(self, rhs: Self) -> Self {
                    self.wrapping_mul(rhs)
                }
            }

            impl $crate::ops::WrappingMulAssign for $ty {
                fn wrapping_mul_assign(&mut self, rhs: Self) {
                    *self = self.wrapping_mul(rhs);
                }
            }
        )*
    }
}

impl_wrapping_mul!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
