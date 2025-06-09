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
    ($left:ty, $right:ty, $func:ident) => {};
    (@nightly $left:ty, $right:ty, $func:ident) => {};
}
