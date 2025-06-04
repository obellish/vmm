pub trait GetOrZero<T> {
	fn get_or_zero(self) -> T;
}

macro_rules! impl_get_or_zero {
    ($($ty:ty)*) => {
        $(
            impl $crate::GetOrZero<$ty> for ::core::option::Option<::core::num::NonZero<$ty>> {
                #[inline]
                fn get_or_zero(self) -> $ty {
                    match self {
                        ::core::option::Option::None => 0,
                        ::core::option::Option::Some(v) => v.get(),
                    }
                }
            }

            impl $crate::GetOrZero<$ty> for ::core::num::NonZero<$ty> {
                #[inline]
                fn get_or_zero(self) -> $ty {
                    ::core::num::NonZero::get(self)
                }
            }
        )*
    };
}

impl_get_or_zero!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
