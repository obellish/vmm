pub trait GetOrZero<T> {
	fn get_or_zero(self) -> T;
}

macro_rules! impl_get_or_zero {
    ($($ty:ty)*) => {
        $(
            impl $crate::GetOrZero<$ty> for ::core::option::Option<::core::num::NonZero<$ty>> {
                fn get_or_zero(self) -> $ty {
                    ::core::option::Option::map_or(self, 0, ::core::num::NonZero::get)
                }
            }
        )*
    };
}

impl_get_or_zero!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
