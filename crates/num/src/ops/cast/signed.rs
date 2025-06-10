use super::CastTo;

pub trait CastToI8: CastTo<i8> {}

impl<T> CastToI8 for T where T: CastTo<i8> {}

pub trait CastToI16: CastTo<i16> {}

impl<T> CastToI16 for T where T: CastTo<i16> {}

pub trait CastToI32: CastTo<i32> {}

impl<T> CastToI32 for T where T: CastTo<i32> {}

pub trait CastToI64: CastTo<i64> {}

impl<T> CastToI64 for T where T: CastTo<i64> {}

pub trait CastToI128: CastTo<i128> {}

impl<T> CastToI128 for T where T: CastTo<i128> {}

pub trait CastToIsize: CastTo<isize> {}

impl<T> CastToIsize for T where T: CastTo<isize> {}
