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

            impl $crate::GetOrZero<$ty> for ::core::option::Option<$ty> {
                fn get_or_zero(self) -> $ty {
                    match self {
                        ::core::option::Option::None => 0,
                        ::core::option::Option::Some(v) => v,
                    }
                }
            }

            impl $crate::GetOrZero<$ty> for $ty {
                fn get_or_zero(self) -> $ty {
                    self
                }
            }
        )*
    };
}

impl_get_or_zero!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);

#[cfg(test)]
mod tests {
	use core::num::NonZero;

	// use quickcheck::{TestResult, quickcheck};
	use vmm_testing::run_test;

	use super::GetOrZero;

	// fn u8_check(value: u8) -> TestResult {
	// 	TestResult::from_bool(NonZero::new(value).get_or_zero() == value)
	// }

	// fn u16_check(value: u16) -> TestResult {
	// 	TestResult::from_bool(NonZero::new(value).get_or_zero() == value)
	// }

	// #[test]
	// fn unsigned_works() {
	// 	quickcheck(u8_check as fn(u8) -> TestResult);
	// 	quickcheck(u16_check as fn(u16) -> TestResult);
	// }

	#[test]
	fn unsigned_works() {
		run_test(|u| {
			let value = u.arbitrary::<u8>()?;

			assert_eq!(NonZero::new(value).get_or_zero(), value);

			Ok(())
		});

		run_test(|u| {
			let value = u.arbitrary::<u16>()?;

			assert_eq!(NonZero::new(value).get_or_zero(), value);

			Ok(())
		});

		run_test(|u| {
			let value = u.arbitrary::<u32>()?;

			assert_eq!(NonZero::new(value).get_or_zero(), value);

			Ok(())
		});

		run_test(|u| {
			let value = u.arbitrary::<u64>()?;

			assert_eq!(NonZero::new(value).get_or_zero(), value);

			Ok(())
		});

		run_test(|u| {
			let value = u.arbitrary::<usize>()?;

			assert_eq!(NonZero::new(value).get_or_zero(), value);

			Ok(())
		});
	}
}
