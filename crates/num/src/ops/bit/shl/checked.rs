pub trait CheckedShl<Rhs = Self> {
	type Output;

	fn checked_shl(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait CheckedShlAssign<Rhs = Self> {
	fn checked_shl_assign(&mut self, rhs: Rhs);
}
