mod impls;

use alloc::vec::Vec;
use core::{
	fmt::{Binary, Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	num::ParseFloatError,
	ops::{Neg, Not, Rem},
	str::FromStr,
};

pub(super) use self::impls::var_float_to_primitive;
use super::{VarInt, VarUInt};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum VarFloat {
	F32(f32),
	F64(f64),
}

impl VarFloat {
	#[must_use]
	pub const fn reverse_bits(self) -> Self {
		match self {
			Self::F32(f) => Self::F32(f32::from_bits(f.to_bits().reverse_bits())),
			Self::F64(f) => Self::F64(f64::from_bits(f.to_bits().reverse_bits())),
		}
	}

	#[must_use]
	pub const fn zero() -> Self {
		Self::F32(0.)
	}

	#[must_use]
	pub const fn one() -> Self {
		Self::F32(1.)
	}

	#[must_use]
	pub const fn is_one(self) -> bool {
		matches!(self, Self::F32(1.) | Self::F64(1.))
	}

	#[must_use]
	pub const fn is_zero(self) -> bool {
		matches!(self, Self::F32(0.) | Self::F64(0.))
	}

	#[must_use]
	pub const fn is_non_zero(self) -> bool {
		!self.is_zero()
	}

	#[must_use]
	pub fn to_int(self) -> VarInt {
		match self {
			Self::F32(v) => (v as i32).into(),
			Self::F64(v) => (v as i64).into(),
		}
	}

	#[must_use]
	pub fn to_uint(self) -> VarUInt {
		match self {
			Self::F32(v) => (v as u32).into(),
			Self::F64(v) => (v as u64).into(),
		}
	}

	#[must_use]
	pub fn upgrade(self) -> Self {
		match self {
			Self::F32(v) => Self::F64(v.into()),
			Self::F64(v) => panic!("cannot upgrade f64 value: {v}"),
		}
	}

	#[must_use]
	pub fn downgrade(self) -> Self {
		match self {
			Self::F32(v) => panic!("cannot downgrade f32 value: {v}"),
			Self::F64(v) => Self::F32(v as f32),
		}
	}

	#[must_use]
	pub fn to_bytes(self) -> Vec<u8> {
		match self {
			Self::F32(v) => v.to_be_bytes().to_vec(),
			Self::F64(v) => v.to_be_bytes().to_vec(),
		}
	}

	#[must_use]
	pub fn from_bytes(raw: &[u8]) -> Self {
		match raw.len() {
			4 => {
				let mut bytes = [0; 4];
				bytes.copy_from_slice(raw);
				Self::F32(f32::from_be_bytes(bytes))
			}
			8 => {
				let mut bytes = [0; 8];
				bytes.copy_from_slice(raw);
				Self::F64(f64::from_be_bytes(bytes))
			}
			l => panic!("invalid byte length: {l}"),
		}
	}

	#[must_use]
	pub fn normalize(lhs: Self, rhs: Self) -> (Self, Self) {
		match (lhs, rhs) {
			(Self::F32(lhs), Self::F32(rhs)) => (Self::F32(lhs), Self::F32(rhs)),
			(Self::F32(lhs), rhs) => Self::normalize(Self::F32(lhs).upgrade(), rhs),
			(lhs, Self::F32(rhs)) => Self::normalize(lhs, Self::F32(rhs).upgrade()),
			(Self::F64(lhs), Self::F64(rhs)) => (Self::F64(lhs), Self::F64(rhs)),
		}
	}
}

impl Binary for VarFloat {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::F32(v) => Binary::fmt(&v.to_bits(), f),
			Self::F64(v) => Binary::fmt(&v.to_bits(), f),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VarFloat {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<f64 as serde::Deserialize>::deserialize(deserializer).map(Into::into)
	}
}

impl Display for VarFloat {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::F32(v) => Display::fmt(&v, f),
			Self::F64(v) => Display::fmt(&v, f),
		}
	}
}

impl Eq for VarFloat {}

impl FromStr for VarFloat {
	type Err = ParseFloatError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.parse::<f64>().map(Into::into)
	}
}

impl Hash for VarFloat {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Self::F32(f) => state.write_u32(f.to_bits()),
			Self::F64(f) => state.write_u64(f.to_bits()),
		}
	}
}

impl Neg for VarFloat {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Self::F32(v) => Self::F32(-v),
			Self::F64(v) => Self::F64(-v),
		}
	}
}

impl Not for VarFloat {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::F32(v) => (!(v as u32)).into(),
			Self::F64(v) => (!(v as u64)).into(),
		}
	}
}

impl Rem for VarFloat {
	type Output = Self;

	fn rem(self, rhs: Self) -> Self::Output {
		let (lhs, rhs) = Self::normalize(self, rhs);
		match (lhs, rhs) {
			(Self::F32(lhs), Self::F32(rhs)) => ((lhs as u32) % (rhs as u32)).into(),
			(Self::F64(lhs), Self::F64(rhs)) => ((lhs as u64) % (rhs as u64)).into(),
			(lhs, rhs) => panic!("normalization failed: {lhs} % {rhs}"),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for VarFloat {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_f64(match *self {
			Self::F32(v) => v.into(),
			Self::F64(v) => v,
		})
	}
}
