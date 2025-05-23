pub trait WrappingSub<Rhs = Self> {
	type Output;

	fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_wrapping_sub {
	($signed:ty, $unsigned:ty) => {
		impl $crate::ops::WrappingSub for $signed {
			type Output = Self;

			fn wrapping_sub(self, rhs: Self) -> Self::Output {
				self.wrapping_sub(rhs)
			}
		}

		impl $crate::ops::WrappingSub<$unsigned> for $signed {
			type Output = Self;

			fn wrapping_sub(self, rhs: $unsigned) -> Self::Output {
				self.wrapping_sub_unsigned(rhs)
			}
		}

		impl $crate::ops::WrappingSub for $unsigned {
			type Output = Self;

			fn wrapping_sub(self, rhs: Self) -> Self::Output {
				self.wrapping_sub(rhs)
			}
		}
	};
}

impl_wrapping_sub!(i8, u8);
impl_wrapping_sub!(i16, u16);
impl_wrapping_sub!(i32, u32);
impl_wrapping_sub!(i64, u64);
impl_wrapping_sub!(i128, u128);
impl_wrapping_sub!(isize, usize);
