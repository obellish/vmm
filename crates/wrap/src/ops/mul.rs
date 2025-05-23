pub trait WrappingMul<Rhs = Self> {
	type Output;

	fn wrapping_mul(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingMulAssign<Rhs = Self> {
    fn wrapping_mul_assign(&mut self, rhs: Rhs);
}

macro_rules! impl_wrapping_mul {
    ($signed:ty, $unsigned:ty) => {

    };
}
