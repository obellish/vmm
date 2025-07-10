mod de;

use serde::de::{IntoDeserializer, Unexpected};

pub use self::de::*;
use super::{Error, Result};
use crate::Format;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Unit,
	Bool(bool),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	U128(u128),
	F32(f32),
	F64(f64),
	Char(char),
	Str(String),
	Bytes(Vec<u8>),
	Option(Option<Box<Self>>),
	Variant(u32, Box<Self>),
	Seq(Vec<Self>),
}

impl Value {
	pub(crate) fn seq_values(&self) -> Result<&Vec<Value>> {
		match self {
			Self::Seq(x) => Ok(x),
			_ => Err(Error::Deserialization("seq_values")),
		}
	}

	pub(crate) fn into_format_and_value(self) -> (Format, Self) {
		let format = match self {
			Self::Bool(..) => Format::Bool,
			Self::I8(..) => Format::I8,
			Self::I16(..) => Format::I16,
			Self::I32(..) => Format::I32,
			Self::I64(..) => Format::I64,
			Self::I128(..) => Format::I128,
			Self::U8(..) => Format::U8,
			Self::U16(..) => Format::U16,
			Self::U32(..) => Format::U32,
			Self::U64(..) => Format::U64,
			_ => todo!(),
		};

		(format, self)
	}
}

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Self::Bool(value)
	}
}

impl From<i8> for Value {
	fn from(value: i8) -> Self {
		Self::I8(value)
	}
}

impl From<i16> for Value {
	fn from(value: i16) -> Self {
		Self::I16(value)
	}
}

impl From<i32> for Value {
	fn from(value: i32) -> Self {
		Self::I32(value)
	}
}

impl From<i64> for Value {
	fn from(value: i64) -> Self {
		Self::I64(value)
	}
}

impl From<i128> for Value {
	fn from(value: i128) -> Self {
		Self::I128(value)
	}
}

impl From<u8> for Value {
	fn from(value: u8) -> Self {
		Self::U8(value)
	}
}

impl From<u16> for Value {
	fn from(value: u16) -> Self {
		Self::U16(value)
	}
}

impl From<u32> for Value {
	fn from(value: u32) -> Self {
		Self::U32(value)
	}
}

impl From<u64> for Value {
	fn from(value: u64) -> Self {
		Self::U64(value)
	}
}

impl From<u128> for Value {
	fn from(value: u128) -> Self {
		Self::U128(value)
	}
}

impl From<f32> for Value {
	fn from(value: f32) -> Self {
		Self::F32(value)
	}
}

impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Self::F64(value)
	}
}

impl From<char> for Value {
	fn from(value: char) -> Self {
		Self::Char(value)
	}
}

impl From<String> for Value {
	fn from(value: String) -> Self {
		Self::Str(value)
	}
}

impl From<Vec<u8>> for Value {
	fn from(value: Vec<u8>) -> Self {
		Self::Bytes(value)
	}
}

impl From<()> for Value {
	fn from((): ()) -> Self {
		Self::Unit
	}
}

impl<'a> From<&'a Value> for Unexpected<'a> {
	fn from(value: &'a Value) -> Self {
		match value {
			Value::Unit => Self::Unit,
			Value::Bool(b) => Self::Bool(*b),
			Value::I8(i) => Self::Signed((*i).into()),
			Value::I16(i) => Self::Signed((*i).into()),
			Value::I32(i) => Self::Signed((*i).into()),
			Value::I64(i) => Self::Signed(*i),
			Value::I128(i) => Self::Signed((*i) as i64),
			Value::U8(i) => Self::Unsigned((*i).into()),
			Value::U16(i) => Self::Unsigned((*i).into()),
			Value::U32(i) => Self::Unsigned((*i).into()),
			Value::U64(i) => Self::Unsigned(*i),
			Value::U128(i) => Self::Unsigned((*i) as u64),
			Value::F32(f) => Self::Float((*f).into()),
			Value::F64(f) => Self::Float(*f),
			Value::Char(c) => Self::Char(*c),
			Value::Str(s) => Self::Str(s),
			Value::Bytes(b) => Self::Bytes(b),
			Value::Option(..) => Self::Option,
			Value::Variant(..) => Self::Enum,
			Value::Seq(..) => Self::Seq,
		}
	}
}

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
	type Deserializer = ValueDeserializer<'de>;

	fn into_deserializer(self) -> Self::Deserializer {
		ValueDeserializer::new(self)
	}
}
