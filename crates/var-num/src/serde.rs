use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, Visitor},
};

use super::VarNum;

impl<'de> Deserialize<'de> for VarNum {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(VarNumVisitor)
	}
}

impl Serialize for VarNum {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Int(v) => v.serialize(serializer),
			Self::UInt(v) => v.serialize(serializer),
			Self::Float(v) => v.serialize(serializer),
		}
	}
}

struct VarNumVisitor;

impl Visitor<'_> for VarNumVisitor {
	type Value = VarNum;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a VarNum variant")
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(VarNum::Int(v.into()))
	}

	fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(VarNum::Int(v.into()))
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(VarNum::UInt(v.into()))
	}

	fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(VarNum::UInt(v.into()))
	}

	fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(VarNum::Float(v.into()))
	}
}
