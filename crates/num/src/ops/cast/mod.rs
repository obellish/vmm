mod signed;
mod unsigned;

pub use self::{signed::*, unsigned::*};

pub trait CastTo<T: Primitive> {
	fn cast_to(self) -> T;
}

impl CastTo<u8> for char {
	fn cast_to(self) -> u8 {
		self as u8
	}
}

impl CastTo<char> for u8 {
	fn cast_to(self) -> char {
		self as char
	}
}

macro_rules! impl_cast_to {
    ($to:ty, $($ty:ty)*) => {
        $(
            #[allow(clippy::cast_lossless)]
            impl $crate::ops::CastTo<$to> for $ty {
                fn cast_to(self) -> $to {
                    self as $to
                }
            }
        )*
    };
}

impl_cast_to!(u8, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(u16, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(u32, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(u64, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(u128, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(usize, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(i8, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(i16, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(i32, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(i64, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(i128, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);
impl_cast_to!(isize, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

pub unsafe trait Primitive {}

macro_rules! impl_primitive {
    ($($ty:ty)*) => {
        $(
            unsafe impl $crate::ops::Primitive for $ty {}
        )*
    };
}

impl_primitive!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize char);
