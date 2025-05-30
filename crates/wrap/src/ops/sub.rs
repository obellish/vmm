pub trait WrappingSub<Rhs = Self> {
	type Output;

	fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingSubAssign<Rhs = Self> {
	fn wrapping_sub_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_sub {
	($signed:ty, $unsigned:ty) => {
		impl_wrapping_sub!($signed, $signed, wrapping_sub);
		impl_wrapping_sub!($unsigned, $unsigned, wrapping_sub);
		impl_wrapping_sub!($signed, $unsigned, wrapping_sub_unsigned);
		impl_wrapping_sub!(@nightly $unsigned, $signed, wrapping_sub_signed);
	};
	($left:ty, $right:ty, $func:ident) => {
		impl $crate::ops::WrappingSub<$right> for $left {
			type Output = $left;

			fn wrapping_sub(self, rhs: $right) -> Self {
				<$left>::$func(self, rhs)
			}
		}

		impl $crate::ops::WrappingSub<$right> for &$left {
			type Output = $left;

			fn wrapping_sub(self, rhs: $right) -> $left {
				<$left>::$func(*self, rhs)
			}
		}

		impl $crate::ops::WrappingSub<&$right> for $left {
			type Output = $left;

			fn wrapping_sub(self, rhs: &$right) -> Self {
				<$left>::$func(self, *rhs)
			}
		}

		impl $crate::ops::WrappingSub<&$right> for &$left {
			type Output = $left;

			fn wrapping_sub(self, rhs: &$right) -> $left {
				<$left>::$func(*self, *rhs)
			}
		}
	};
	(@nightly $left:ty, $right:ty, $func:ident) => {
		#[cfg(feature = "nightly")]
		impl $crate::ops::WrappingSub<$right> for $left {
			type Output = $left;

			fn wrapping_sub(self, rhs: $right) -> Self {
				<$left>::$func(self, rhs)
			}
		}

		#[cfg(feature = "nightly")]
		impl $crate::ops::WrappingSub<$right> for &$left {
			type Output = $left;

			fn wrapping_sub(self, rhs: $right) -> $left {
				<$left>::$func(*self, rhs)
			}
		}

		#[cfg(feature = "nightly")]
		impl $crate::ops::WrappingSub<&$right> for $left {
			type Output = $left;

			fn wrapping_sub(self, rhs: &$right) -> Self {
				<$left>::$func(self, *rhs)
			}
		}

		#[cfg(feature = "nightly")]
		impl $crate::ops::WrappingSub<&$right> for &$left {
			type Output = $left;

			fn wrapping_sub(self, rhs: &$right) -> $left {
				<$left>::$func(*self, *rhs)
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
