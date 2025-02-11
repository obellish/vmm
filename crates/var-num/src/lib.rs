#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod impls;
mod inner;
#[cfg(feature = "serde")]
mod serde;

use alloc::vec::Vec;
use core::{
	fmt::{Binary, Display, Formatter, Result as FmtResult},
	num::{ParseFloatError, ParseIntError},
};

use thiserror::Error;

pub use self::inner::{VarFloat, VarInt, VarUInt};

#[derive(Debug, Error)]
pub enum ParseNumError {
	#[error(transparent)]
	ParseFloat(#[from] ParseFloatError),
	#[error(transparent)]
	ParseInt(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum VarNum {
	Float(VarFloat),
	Int(VarInt),
	UInt(VarUInt),
}

impl VarNum {
	#[must_use]
	pub const fn zero() -> Self {
		Self::Int(VarInt::zero())
	}

	#[must_use]
	pub const fn one() -> Self {
		Self::Int(VarInt::one())
	}

	#[must_use]
	pub const fn is_one(self) -> bool {
		match self {
			Self::Float(v) => v.is_one(),
			Self::Int(v) => v.is_one(),
			Self::UInt(v) => v.is_one(),
		}
	}

	#[must_use]
	pub const fn is_zero(self) -> bool {
		match self {
			Self::Float(v) => v.is_zero(),
			Self::Int(v) => v.is_zero(),
			Self::UInt(v) => v.is_zero(),
		}
	}

	#[must_use]
	pub const fn is_non_zero(self) -> bool {
		!self.is_zero()
	}

	#[must_use]
	pub const fn reverse_bits(self) -> Self {
		match self {
			Self::Float(v) => Self::Float(v.reverse_bits()),
			Self::Int(v) => Self::Int(v.reverse_bits()),
			Self::UInt(v) => Self::UInt(v.reverse_bits()),
		}
	}

	#[must_use]
	pub fn to_bytes(self) -> Vec<u8> {
		let mut bytes = vec![];
		match self {
			Self::Int(i) => {
				bytes.push(1);
				bytes.extend(i.to_bytes());
			}
			Self::UInt(u) => {
				bytes.push(2);
				bytes.extend(u.to_bytes());
			}
			Self::Float(f) => {
				bytes.push(3);
				bytes.extend(f.to_bytes());
			}
		}

		bytes
	}

	#[must_use]
	pub fn from_bytes(raw: &[u8]) -> Self {
		match raw[0] {
			1 => Self::Int(VarInt::from_bytes(&raw[1..])),
			2 => Self::UInt(VarUInt::from_bytes(&raw[1..])),
			3 => Self::Float(VarFloat::from_bytes(&raw[1..])),
			b => panic!("invalid VarNum type: {b}"),
		}
	}

	#[must_use]
	pub fn upgrade(self) -> Self {
		match self {
			Self::Int(i) => Self::Int(i.upgrade()),
			Self::UInt(u) => Self::UInt(u.upgrade()),
			Self::Float(f) => Self::Float(f.upgrade()),
		}
	}

	#[must_use]
	pub fn downgrade(self) -> Self {
		match self {
			Self::Int(i) => Self::Int(i.downgrade()),
			Self::UInt(u) => Self::UInt(u.downgrade()),
			Self::Float(f) => Self::Float(f.downgrade()),
		}
	}

	#[must_use]
	pub const fn to_float(self) -> Self {
		match self {
			Self::Int(i) => Self::Float(i.to_float()),
			Self::UInt(u) => Self::Float(u.to_float()),
			Self::Float(f) => Self::Float(f),
		}
	}

	#[must_use]
	pub fn to_int(self) -> Self {
		match self {
			Self::Int(i) => Self::Int(i),
			Self::UInt(u) => Self::Int(u.to_int()),
			Self::Float(f) => Self::Int(f.to_int()),
		}
	}

	#[must_use]
	pub fn to_uint(self) -> Self {
		match self {
			Self::Int(i) => Self::UInt(i.to_uint()),
			Self::UInt(u) => Self::UInt(u),
			Self::Float(f) => Self::UInt(f.to_uint()),
		}
	}

	#[must_use]
	pub fn normalize(lhs: Self, rhs: Self) -> (Self, Self) {
		let (lhs, rhs) = match (lhs, rhs) {
			(lhs @ Self::Float(_), rhs @ Self::Float(_))
			| (lhs @ Self::Int(_), rhs @ Self::Int(_))
			| (lhs @ Self::UInt(_), rhs @ Self::UInt(_)) => (lhs, rhs),
			(lhs @ Self::Float(_), rhs) => (lhs, rhs.to_float()),
			(lhs @ Self::Int(_), rhs) => (lhs, rhs.to_int()),
			(lhs @ Self::UInt(_), rhs) => (lhs, rhs.to_uint()),
		};

		match (lhs, rhs) {
			(Self::Float(lhs), Self::Float(rhs)) => {
				let (lhs, rhs) = VarFloat::normalize(lhs, rhs);
				(Self::Float(lhs), Self::Float(rhs))
			}
			(Self::Int(lhs), Self::Int(rhs)) => {
				let (lhs, rhs) = VarInt::normalize(lhs, rhs);
				(Self::Int(lhs), Self::Int(rhs))
			}
			(Self::UInt(lhs), Self::UInt(rhs)) => {
				let (lhs, rhs) = VarUInt::normalize(lhs, rhs);
				(Self::UInt(lhs), Self::UInt(rhs))
			}
			(lhs, rhs) => panic!("normalization failed: {lhs} + {rhs}"),
		}
	}
}

impl Binary for VarNum {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Int(i) => Binary::fmt(&i, f),
			Self::UInt(u) => Binary::fmt(&u, f),
			Self::Float(fl) => Binary::fmt(&fl, f),
		}
	}
}

impl Display for VarNum {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Float(v) => Display::fmt(&v, f),
			Self::Int(v) => Display::fmt(&v, f),
			Self::UInt(v) => Display::fmt(&v, f),
		}
	}
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
	use super::{VarInt, VarNum};

	#[test]
	fn it_works() {
		let a: VarNum = 2.into();
		let b: VarNum = 2.0.into();
		let c: VarNum = a + b;
		assert_eq!(c, VarNum::Int(VarInt::I8(4)));
	}

	macro_rules! generate_unary_tests {
		($($group_name:ident > $ty:ty: $op:tt),*) => {
			$(
				#[test]
				fn $group_name() {
					let a: $ty = <$ty>::MAX / (2 as $ty);
					let a_n: $crate::VarNum = a.into();
					assert_eq!($op a, ($op a_n).into());
				}
			)*
		};
	}

	macro_rules! generate_binary_tests {
		($($group_name:ident > $ty:ty: $op:tt),*) => {
			$(
				#[test]
				fn $group_name() {
					let a: $ty = 6 as $ty;
					let b: $ty = 2 as $ty;
					let a_n: $crate::VarNum = a.into();
					let b_n: $crate::VarNum = b.into();
					assert_eq!(a $op b, (a_n $op b_n).into());
				}
			)*
		};
	}

	generate_unary_tests! {
		u8_not > u8: !,
		u16_not > u16: !,
		u32_not > u32: !,
		u64_not > u64: !,
		u128_not > u128: !,
		i8_not > i8: !,
		i16_not > i16: !,
		i32_not > i32: !,
		i64_not > i64: !,
		i128_not > i128: !,
		i8_neg > i8: -,
		i16_neg > i16: -,
		i32_neg > i32: -,
		i64_neg > i64: -,
		i128_neg > i128: -,
		f32_neg > f32: -,
		f64_neg > f64: -
	}

	generate_binary_tests! {
		u8_add > u8: +,
		f32_add > f32: +,
		f64_add > f64: +,
		u64_sub > u64: -,
		u128_sub > u128: -,
		i8_sub > i8: -,
		i16_sub > i16: -,
		u16_mul > u16: *,
		u32_mul > u32: *,
		u64_mul > u64: *,
		u128_mul > u128: *,
		i8_mul > i8: *,
		i16_mul > i16: *,
		f64_mul > f64: *,
		u128_div > u128: /,
		i8_div > i8: /,
		i32_div > i32: /,
		f64_div > f64: /,
		u64_rem > u64: %,
		i16_rem > i16: %,
		i32_rem > i32: %
	}
}
