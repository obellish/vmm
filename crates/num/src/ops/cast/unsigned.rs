use super::CastTo;

pub trait CastToU8: CastTo<u8> {}

impl<T> CastToU8 for T where T: CastTo<u8> {}

pub trait CastToU16: CastTo<u16> {}

impl<T> CastToU16 for T where T: CastTo<u16> {}

pub trait CastToU32: CastTo<u32> {}

impl<T> CastToU32 for T where T: CastTo<u32> {}

pub trait CastToU64: CastTo<u64> {}

impl<T> CastToU64 for T where T: CastTo<u64> {}

pub trait CastToU128: CastTo<u128> {}

impl<T> CastToU128 for T where T: CastTo<u128> {}

pub trait CastToUsize: CastTo<usize> {}

impl<T> CastToUsize for T where T: CastTo<usize> {}
