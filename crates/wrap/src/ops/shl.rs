pub trait WrappingShl<Rhs = Self> {
	type Output;

	fn wrapping_shl(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingShlAssign<Rhs = Self> {
	fn wrapping_shl_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_shl {
	($($ty:ty)*) => {
		$(
			impl $crate::ops::WrappingShl<u32> for $ty {
				type Output = Self;

				fn wrapping_shl(self, rhs: u32) -> Self {
					<$ty>::wrapping_shl(self, rhs)
				}
			}

			impl $crate::ops::WrappingShl<&u32> for $ty {
				type Output = Self;

				fn wrapping_shl(self, rhs: &u32) -> Self {
					<$ty>::wrapping_shl(self, *rhs)
				}
			}

			impl $crate::ops::WrappingShl<u32> for &$ty {
				type Output = $ty;

				fn wrapping_shl(self, rhs: u32) -> $ty {
					<$ty>::wrapping_shl(*self, rhs)
				}
			}

			impl $crate::ops::WrappingShl<&u32> for &$ty {
				type Output = $ty;

				fn wrapping_shl(self, rhs: &u32) -> $ty {
					<$ty>::wrapping_shl(*self, *rhs)
				}
			}

			impl $crate::ops::WrappingShlAssign<u32> for $ty {
				fn wrapping_shl_assign(&mut self, rhs: u32) {
					*self = <$ty>::wrapping_shl(*self, rhs);
				}
			}

			impl $crate::ops::WrappingShlAssign<&u32> for $ty {
				fn wrapping_shl_assign(&mut self, rhs: &u32) {
					*self = <$ty>::wrapping_shl(*self, *rhs);
				}
			}
		)*
	};
}

impl_wrapping_shl!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);
