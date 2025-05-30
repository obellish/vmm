pub trait WrappingSub<Rhs = Self> {
	type Output;

	fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingSubAssign<Rhs = Self> {
	fn wrapping_sub_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_sub {
	($signed:ty, $unsigned:ty) => {
		impl $crate::ops::WrappingSub for $signed {
			type Output = Self;

			fn wrapping_sub(self, rhs: Self) -> Self::Output {
				self.wrapping_sub(rhs)
			}
		}

		impl $crate::ops::WrappingSub for &$signed {
			type Output = $signed;

			fn wrapping_sub(self, rhs: Self) -> $signed {
				<$signed>::wrapping_sub(*self, *rhs)
			}
		}

		impl $crate::ops::WrappingSub<$unsigned> for $signed {
			type Output = Self;

			fn wrapping_sub(self, rhs: $unsigned) -> Self::Output {
				self.wrapping_sub_unsigned(rhs)
			}
		}

		impl $crate::ops::WrappingSub<$unsigned> for &$signed {
			type Output = $signed;

			fn wrapping_sub(self, rhs: $unsigned) -> Self::Output {
				<$signed>::wrapping_sub_unsigned(*self, rhs)
			}
		}

		impl $crate::ops::WrappingSub<&$unsigned> for $signed {
			type Output = Self;

			fn wrapping_sub(self, rhs: &$unsigned) -> Self::Output {
				self.wrapping_sub_unsigned(*rhs)
			}
		}

		impl $crate::ops::WrappingSub<&$unsigned> for &$signed {
			type Output = $signed;

			fn wrapping_sub(self, rhs: &$unsigned) -> Self::Output {
				<$signed>::wrapping_sub_unsigned(*self, *rhs)
			}
		}

		impl $crate::ops::WrappingSub for $unsigned {
			type Output = Self;

			fn wrapping_sub(self, rhs: Self) -> Self::Output {
				self.wrapping_sub(rhs)
			}
		}

		impl $crate::ops::WrappingSub for &$unsigned {
			type Output = $unsigned;

			fn wrapping_sub(self, rhs: Self) -> $unsigned {
				<$unsigned>::wrapping_sub(*self, *rhs)
			}
		}

		#[cfg(feature = "nightly")]
		impl $crate::ops::WrappingSub<$signed> for $unsigned {
			type Output = Self;

			fn wrapping_sub(self, rhs: $signed) -> Self::Output {
				self.wrapping_sub_signed(rhs)
			}
		}

		impl $crate::ops::WrappingSubAssign for $signed {
			fn wrapping_sub_assign(&mut self, rhs: Self) {
				*self = self.wrapping_sub(rhs);
			}
		}

		impl $crate::ops::WrappingSubAssign<$unsigned> for $signed {
			fn wrapping_sub_assign(&mut self, rhs: $unsigned) {
				*self = self.wrapping_sub_unsigned(rhs);
			}
		}

		impl $crate::ops::WrappingSubAssign for $unsigned {
			fn wrapping_sub_assign(&mut self, rhs: Self) {
				*self = self.wrapping_sub(rhs);
			}
		}

		#[cfg(feature = "nightly")]
		impl $crate::ops::WrappingSubAssign<$signed> for $unsigned {
			fn wrapping_sub_assign(&mut self, rhs: $signed) {
				*self = self.wrapping_sub_signed(rhs);
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
