pub trait CastToU8 {
	fn cast(self) -> u8;
}

pub trait CastToU16 {
	fn cast(self) -> u16;
}

pub trait CastToU32 {
	fn cast(self) -> u32;
}

pub trait CastToU64 {
	fn cast(self) -> u64;
}

pub trait CastToU128 {
	fn cast(self) -> u128;
}

pub trait CastToUsize {
	fn cast(self) -> usize;
}

macro_rules! impl_cast_to_unsigned {
	($t:ident => $v:ty) => {
		impl<T> $crate::ops::$t for T
		where
			T: $crate::ops::CastTo<$v>,
		{
			fn cast(self) -> $v {
				$crate::ops::CastTo::cast(self)
			}
		}
	};
}

impl_cast_to_unsigned!(CastToU8 => u8);
impl_cast_to_unsigned!(CastToU16 => u16);
impl_cast_to_unsigned!(CastToU32 => u32);
impl_cast_to_unsigned!(CastToU64 => u64);
impl_cast_to_unsigned!(CastToU128 => u128);
impl_cast_to_unsigned!(CastToUsize => usize);
