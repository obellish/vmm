mod de;
mod float;
mod integer;
mod ser;
#[cfg(test)]
mod tests;

use alloc::{
	borrow::{Cow, ToOwned},
	boxed::Box,
	collections::VecDeque,
	string::String,
	vec::Vec,
};
use core::{
	fmt::Write,
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize, de::Unexpected};

pub use self::{de::*, float::*, integer::*, ser::*};
use super::{Config, Result};

#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct OwnedValue(Value<'static>);

#[derive(Debug, Default, Clone)]
pub enum Value<'a> {
	#[default]
	Null,
	Bool(bool),
	Integer(Integer),
	Float(Float),
	Bytes(Cow<'a, [u8]>),
	String(Cow<'a, str>),
	Array(VecDeque<Self>),
	Map(VecDeque<(Self, Self)>),
}

impl<'a> Value<'a> {
	pub fn reborrow(&self) -> Value<'_> {
		match self {
			Self::Null => Self::Null,
			Self::Bool(b) => Self::Bool(*b),
			Self::Integer(int) => Self::Integer(*int),
			Self::Float(f) => Self::Float(*f),
			Self::Bytes(bytes) => Value::Bytes(Cow::Borrowed(bytes)),
			Self::String(s) => Value::String(Cow::Borrowed(s)),
			Self::Array(arr) => Value::Array(arr.iter().map(Self::reborrow).collect()),
			Self::Map(map) => Value::Map(
				map.iter()
					.map(|(key, value)| (key.reborrow(), value.reborrow()))
					.collect(),
			),
		}
	}

	#[must_use]
	pub fn into_owned(self) -> OwnedValue {
		let value = match self {
			Self::Null => Value::Null,
			Self::Bool(b) => Value::Bool(b),
			Self::Integer(int) => Value::Integer(int),
			Self::Float(float) => Value::Float(float),
			Self::Bytes(bytes) => Value::Bytes(Cow::Owned(bytes.into_owned())),
			Self::String(s) => Value::String(Cow::Owned(s.into_owned())),
			Self::Array(a) => Value::Array(a.into_iter().map(|v| v.into_owned().0).collect()),
			Self::Map(m) => Value::Map(
				m.into_iter()
					.map(|(key, value)| (key.into_owned().0, value.into_owned().0))
					.collect(),
			),
		};

		OwnedValue(value)
	}

	pub fn deserialize_as<'de, T>(self) -> Result<T>
	where
		'a: 'de,
		T: Deserialize<'de>,
	{
		T::deserialize(self::de::ValueDeserializer::new(self))
	}
}

impl<'a, 'de> From<&'a Value<'de>> for Unexpected<'a> {
	fn from(value: &'a Value<'de>) -> Self {
		match value {
			Value::Null => Self::Unit,
			Value::Bool(b) => Self::Bool(*b),
			Value::Integer(Integer::Unsigned(int)) => Self::Unsigned(*int as u64),
			Value::Integer(Integer::Signed(int)) => Self::Signed(*int as i64),
			Value::Float(Float::F32(f)) => Self::Float((*f).into()),
			Value::Float(Float::F64(f)) => Self::Float(*f),
			Value::Bytes(bytes) => Self::Bytes(bytes),
			Value::String(s) => Self::Str(s),
			Value::Array(..) => Self::Seq,
			Value::Map(..) => Self::Map,
		}
	}
}

pub fn from_value_with_config<'de, T>(value: Value<'de>, _: Config) -> Result<T>
where
	T: Deserialize<'de>,
{
	let de = self::de::ValueDeserializer::new(value);
	T::deserialize(de)
}

pub fn from_value<'de, T>(value: Value<'de>) -> Result<T>
where
	T: Deserialize<'de>,
{
	from_value_with_config(value, Config::default())
}
