use core::{
	fmt::{Debug, Display},
	ops::{Add, AddAssign, DivAssign, MulAssign, SubAssign},
};

pub(crate) use num_traits::{Float, FloatConst, NumCast, cast::cast};

pub(crate) use super::euclid::Trig;

pub trait Scalar:
	Float
	+ NumCast
	+ FloatConst
	+ Sized
	+ Display
	+ Debug
	+ Trig
	+ Add
	+ SubAssign
	+ MulAssign
	+ DivAssign
{
	const HALF: Self;
	const ZERO: Self;
	const ONE: Self;
	const TWO: Self;
	const THREE: Self;
	const FOUR: Self;
	const FIVE: Self;
	const SIX: Self;
	const SEVEN: Self;
	const EIGHT: Self;
	const NINE: Self;
	const TEN: Self;

	const MIN: Self;
	const MAX: Self;

	const EPSILON: Self;
	const DIV_EPSILON: Self = Self::EPSILON;

	fn epsilon_for(_reference: Self) -> Self {
		Self::EPSILON
	}

	fn value(v: f32) -> Self;
}

impl Scalar for f32 {
	const EIGHT: Self = 8.0;
	const EPSILON: Self = 1e-4;
	const FIVE: Self = 5.0;
	const FOUR: Self = 4.0;
	const HALF: Self = 0.5;
	const MAX: Self = Self::MAX;
	const MIN: Self = Self::MIN;
	const NINE: Self = 9.0;
	const ONE: Self = 1.0;
	const SEVEN: Self = 7.0;
	const SIX: Self = 6.0;
	const TEN: Self = 10.0;
	const THREE: Self = 3.0;
	const TWO: Self = 2.0;
	const ZERO: Self = 0.0;

	fn epsilon_for(reference: Self) -> Self {
		let magnitude = reference.abs() as i32;
		match magnitude {
			0..=7 => 1e-5,
			8..=1023 => 1e-3,
			1024..=4095 => 1e-2,
			4096..=65535 => 1e-1,
			65536..=8_388_607 => 0.5,
			_ => 1.0,
		}
	}

	fn value(v: f32) -> Self {
		v
	}
}

impl Scalar for f64 {
	const EIGHT: Self = 8.0;
	const EPSILON: Self = 1e-8;
	const FIVE: Self = 5.0;
	const FOUR: Self = 4.0;
	const HALF: Self = 0.5;
	const MAX: Self = Self::MAX;
	const MIN: Self = Self::MIN;
	const NINE: Self = 9.0;
	const ONE: Self = 1.0;
	const SEVEN: Self = 7.0;
	const SIX: Self = 6.0;
	const TEN: Self = 10.0;
	const THREE: Self = 3.0;
	const TWO: Self = 2.0;
	const ZERO: Self = 0.0;

	fn epsilon_for(reference: Self) -> Self {
		let magnitude = reference.abs() as i64;
		match magnitude {
			0..=65535 => 1e-8,
			65536..=8_388_607 => 1e-5,
			8_388_608..=4_294_967_295 => 1e-3,
			_ => 1e-1,
		}
	}

	fn value(v: f32) -> Self {
		v.into()
	}
}
