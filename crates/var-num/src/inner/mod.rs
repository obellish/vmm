mod float;
mod impls;

use core::{
	fmt::{Binary, Display, Formatter, Result as FmtResult},
	num::{ParseIntError, TryFromIntError},
	ops::{Add, Div, Mul, Neg, Not, Sub},
	str::FromStr,
};

pub use self::float::VarFloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VarInt {
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),
}

impl VarInt {
	#[must_use]
	pub const fn reverse_bits(self) -> Self {
		match self {
			Self::I8(v) => Self::I8(v.reverse_bits()),
			Self::I16(v) => Self::I16(v.reverse_bits()),
			Self::I32(v) => Self::I32(v.reverse_bits()),
			Self::I64(v) => Self::I64(v.reverse_bits()),
			Self::I128(v) => Self::I128(v.reverse_bits()),
		}
	}

	#[must_use]
	pub const fn zero() -> Self {
		Self::I8(0)
	}

	#[must_use]
	pub const fn one() -> Self {
		Self::I8(1)
	}

	#[must_use]
	pub const fn is_one(self) -> bool {
		matches!(
			self,
			Self::I8(1) | Self::I16(1) | Self::I32(1) | Self::I64(1) | Self::I128(1)
		)
	}

	#[must_use]
	pub const fn is_zero(self) -> bool {
		matches!(
			self,
			Self::I8(0) | Self::I16(0) | Self::I32(0) | Self::I64(0) | Self::I128(0)
		)
	}

	#[must_use]
	pub const fn is_non_zero(self) -> bool {
		!self.is_zero()
	}

	#[must_use]
	pub const fn to_uint(self) -> VarUInt {
		match self {
			Self::I8(v) => VarUInt::U8(v as u8),
			Self::I16(v) => VarUInt::U16(v as u16),
			Self::I32(v) => VarUInt::U32(v as u32),
			Self::I64(v) => VarUInt::U64(v as u64),
			Self::I128(v) => VarUInt::U128(v as u128),
		}
	}

	#[must_use]
	pub const fn to_float(self) -> VarFloat {
		match self {
			Self::I8(v) => VarFloat::F32(v as f32),
			Self::I16(v) => VarFloat::F32(v as f32),
			Self::I32(v) => VarFloat::F32(v as f32),
			Self::I64(v) => VarFloat::F64(v as f64),
			Self::I128(v) => VarFloat::F64(v as f64),
		}
	}
}

impl Add for VarInt {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let (lhs, rhs) = Self::normalize(self, rhs);
		match (lhs, rhs) {
			(Self::I8(a), Self::I8(b)) => a
				.checked_add(b)
				.map_or_else(|| Self::I16(i16::from(a) + i16::from(b)), Self::I8),
			(Self::I16(a), Self::I16(b)) => a
				.checked_add(b)
				.map_or_else(|| Self::I32(i32::from(a) + i32::from(b)), Self::I16),
			(Self::I32(a), Self::I32(b)) => a
				.checked_add(b)
				.map_or_else(|| Self::I64(i64::from(a) + i64::from(b)), Self::I32),
			(Self::I64(a), Self::I64(b)) => a
				.checked_add(b)
				.map_or_else(|| Self::I128(i128::from(a) + i128::from(b)), Self::I64),
			(Self::I128(a), Self::I128(b)) => a
				.checked_add(b)
				.map_or_else(|| panic!("overflow: {a} + {b}"), Self::I128),
			(lhs, rhs) => panic!("normalization failed: {lhs} + {rhs}"),
		}
	}
}

impl Binary for VarInt {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::I8(v) => Binary::fmt(&v, f),
			Self::I16(v) => Binary::fmt(&v, f),
			Self::I32(v) => Binary::fmt(&v, f),
			Self::I64(v) => Binary::fmt(&v, f),
			Self::I128(v) => Binary::fmt(&v, f),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VarInt {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let i = <i128 as serde::Deserialize>::deserialize(deserializer)?;
		Ok(i.into())
	}
}

impl Display for VarInt {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::I8(v) => Display::fmt(&v, f),
			Self::I16(v) => Display::fmt(&v, f),
			Self::I32(v) => Display::fmt(&v, f),
			Self::I64(v) => Display::fmt(&v, f),
			Self::I128(v) => Display::fmt(&v, f),
		}
	}
}

impl Div for VarInt {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		let (lhs, rhs) = Self::normalize(self, rhs);
		assert!(rhs.is_non_zero(), "cannot divide by zero");

		if lhs.is_zero() {
			return lhs;
		}

		match (lhs, rhs) {
			(Self::I8(a), Self::I8(b)) => a
				.checked_div(b)
				.map_or_else(|| Self::I16(i16::from(a) / i16::from(b)), Self::I8),
			(Self::I16(a), Self::I16(b)) => a
				.checked_div(b)
				.map_or_else(|| Self::I32(i32::from(a) / i32::from(b)), Self::I16),
			(Self::I32(a), Self::I32(b)) => a
				.checked_div(b)
				.map_or_else(|| Self::I64(i64::from(a) / i64::from(b)), Self::I32),
			(Self::I64(a), Self::I64(b)) => a
				.checked_div(b)
				.map_or_else(|| Self::I128(i128::from(a) / i128::from(b)), Self::I64),
			(Self::I128(a), Self::I128(b)) => a
				.checked_div(b)
				.map_or_else(|| panic!("overflow: {a} / {b}"), Self::I128),
			(a, b) => panic!("normalization failed: {a} / {b}"),
		}
	}
}

impl FromStr for VarInt {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let v = s.parse::<i128>()?;
		Ok(v.into())
	}
}

impl Mul for VarInt {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let (lhs, rhs) = Self::normalize(self, rhs);

		if lhs.is_zero() || rhs.is_zero() {
			return Self::I8(0);
		}

		if lhs.is_one() {
			return rhs;
		}

		if rhs.is_one() {
			return lhs;
		}

		match (lhs, rhs) {
			(Self::I8(a), Self::I8(b)) => a
				.checked_mul(b)
				.map_or_else(|| Self::I16(i16::from(a) * i16::from(b)), Self::I8),
			(Self::I16(a), Self::I16(b)) => a
				.checked_mul(b)
				.map_or_else(|| Self::I32(i32::from(a) * i32::from(b)), Self::I16),
			(Self::I32(a), Self::I32(b)) => a
				.checked_mul(b)
				.map_or_else(|| Self::I64(i64::from(a) * i64::from(b)), Self::I32),
			(Self::I64(a), Self::I64(b)) => a
				.checked_mul(b)
				.map_or_else(|| Self::I128(i128::from(a) * i128::from(b)), Self::I64),
			(Self::I128(a), Self::I128(b)) => a
				.checked_mul(b)
				.map_or_else(|| panic!("overflow: {a} * {b}"), Self::I128),
			(lhs, rhs) => panic!("normalization failed: {lhs} * {rhs}"),
		}
	}
}

impl Neg for VarInt {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Self::I8(v) => Self::I8(-v),
			Self::I16(v) => Self::I16(-v),
			Self::I32(v) => Self::I32(-v),
			Self::I64(v) => Self::I64(-v),
			Self::I128(v) => Self::I128(-v),
		}
	}
}

impl Not for VarInt {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::I8(v) => Self::I8(!v),
			Self::I16(v) => Self::I16(!v),
			Self::I32(v) => Self::I32(!v),
			Self::I64(v) => Self::I64(!v),
			Self::I128(v) => Self::I128(!v),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for VarInt {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_i128(match *self {
			Self::I8(v) => v.into(),
			Self::I16(v) => v.into(),
			Self::I32(v) => v.into(),
			Self::I64(v) => v.into(),
			Self::I128(v) => v,
		})
	}
}

impl Sub for VarInt {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		let (lhs, rhs) = Self::normalize(self, rhs);

		if rhs.is_zero() {
			return lhs;
		}

		if lhs.is_zero() {
			return -rhs;
		}

		match (lhs, rhs) {
			(Self::I8(a), Self::I8(b)) => a
				.checked_sub(b)
				.map_or_else(|| Self::I16(i16::from(a) - i16::from(b)), Self::I8),
			(Self::I16(a), Self::I16(b)) => a
				.checked_sub(b)
				.map_or_else(|| Self::I32(i32::from(a) - i32::from(b)), Self::I16),
			(Self::I32(a), Self::I32(b)) => a
				.checked_sub(b)
				.map_or_else(|| Self::I64(i64::from(a) - i64::from(b)), Self::I32),
			(Self::I64(a), Self::I64(b)) => a
				.checked_sub(b)
				.map_or_else(|| Self::I128(i128::from(a) - i128::from(b)), Self::I64),
			(Self::I128(a), Self::I128(b)) => a
				.checked_sub(b)
				.map_or_else(|| panic!("overflow: {a} - {b}"), Self::I128),
			(lhs, rhs) => panic!("normalization failed: {lhs} - {rhs}"),
		}
	}
}

impl TryFrom<VarUInt> for VarInt {
	type Error = TryFromIntError;

	fn try_from(value: VarUInt) -> Result<Self, Self::Error> {
		Ok(match value {
			VarUInt::U8(v) => Self::I8(v.try_into()?),
			VarUInt::U16(v) => Self::I16(v.try_into()?),
			VarUInt::U32(v) => Self::I32(v.try_into()?),
			VarUInt::U64(v) => Self::I64(v.try_into()?),
			VarUInt::U128(v) => Self::I128(v.try_into()?),
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VarUInt {
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	U128(u128),
}

impl VarUInt {
	#[must_use]
	pub const fn reverse_bits(self) -> Self {
		match self {
			Self::U8(v) => Self::U8(v.reverse_bits()),
			Self::U16(v) => Self::U16(v.reverse_bits()),
			Self::U32(v) => Self::U32(v.reverse_bits()),
			Self::U64(v) => Self::U64(v.reverse_bits()),
			Self::U128(v) => Self::U128(v.reverse_bits()),
		}
	}

	#[must_use]
	pub const fn zero() -> Self {
		Self::U8(0)
	}

	#[must_use]
	pub const fn one() -> Self {
		Self::U8(1)
	}

	#[must_use]
	pub const fn is_one(self) -> bool {
		matches!(
			self,
			Self::U8(1) | Self::U16(1) | Self::U32(1) | Self::U64(1) | Self::U128(1)
		)
	}

	#[must_use]
	pub const fn is_zero(self) -> bool {
		matches!(
			self,
			Self::U8(0) | Self::U16(0) | Self::U32(0) | Self::U64(0) | Self::U128(0)
		)
	}

	#[must_use]
	pub const fn is_non_zero(self) -> bool {
		!self.is_zero()
	}

	#[must_use]
	pub const fn to_int(self) -> VarInt {
		match self {
			Self::U8(v) => VarInt::I8(v as i8),
			Self::U16(v) => VarInt::I16(v as i16),
			Self::U32(v) => VarInt::I32(v as i32),
			Self::U64(v) => VarInt::I64(v as i64),
			Self::U128(v) => VarInt::I128(v as i128),
		}
	}

	#[must_use]
	pub const fn to_float(self) -> VarFloat {
		match self {
			Self::U8(v) => VarFloat::F32(v as f32),
			Self::U16(v) => VarFloat::F32(v as f32),
			Self::U32(v) => VarFloat::F32(v as f32),
			Self::U64(v) => VarFloat::F64(v as f64),
			Self::U128(v) => VarFloat::F64(v as f64),
		}
	}
}

impl Binary for VarUInt {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::U8(v) => Binary::fmt(&v, f),
			Self::U16(v) => Binary::fmt(&v, f),
			Self::U32(v) => Binary::fmt(&v, f),
			Self::U64(v) => Binary::fmt(&v, f),
			Self::U128(v) => Binary::fmt(&v, f),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VarUInt {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let i = <u128 as serde::Deserialize>::deserialize(deserializer)?;
		Ok(i.into())
	}
}

impl Display for VarUInt {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::U8(v) => Display::fmt(&v, f),
			Self::U16(v) => Display::fmt(&v, f),
			Self::U32(v) => Display::fmt(&v, f),
			Self::U64(v) => Display::fmt(&v, f),
			Self::U128(v) => Display::fmt(&v, f),
		}
	}
}

impl FromStr for VarUInt {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let v = s.parse::<u128>()?;
		Ok(v.into())
	}
}

impl Neg for VarUInt {
	type Output = Self;

	fn neg(self) -> Self::Output {
		match self {
			Self::U8(v) => Self::U8(!v + 1),
			Self::U16(v) => Self::U16(!v + 1),
			Self::U32(v) => Self::U32(!v + 1),
			Self::U64(v) => Self::U64(!v + 1),
			Self::U128(v) => Self::U128(!v + 1),
		}
	}
}

impl Not for VarUInt {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			Self::U8(v) => Self::U8(!v),
			Self::U16(v) => Self::U16(!v),
			Self::U32(v) => Self::U32(!v),
			Self::U64(v) => Self::U64(!v),
			Self::U128(v) => Self::U128(!v),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for VarUInt {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u128(match *self {
			Self::U8(v) => v.into(),
			Self::U16(v) => v.into(),
			Self::U32(v) => v.into(),
			Self::U64(v) => v.into(),
			Self::U128(v) => v,
		})
	}
}

impl TryFrom<VarInt> for VarUInt {
	type Error = TryFromIntError;

	fn try_from(value: VarInt) -> Result<Self, Self::Error> {
		Ok(match value {
			VarInt::I8(v) => Self::U8(v.try_into()?),
			VarInt::I16(v) => Self::U16(v.try_into()?),
			VarInt::I32(v) => Self::U32(v.try_into()?),
			VarInt::I64(v) => Self::U64(v.try_into()?),
			VarInt::I128(v) => Self::U128(v.try_into()?),
		})
	}
}
