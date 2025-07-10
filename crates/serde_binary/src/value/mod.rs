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
	fmt::{Debug, Formatter, Result as FmtResult},
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use serde::{
	Deserialize, Serialize, Serializer,
	de::{Error as DeError, MapAccess, SeqAccess, Unexpected, Visitor},
	ser::{SerializeMap as _, SerializeSeq as _},
};

pub use self::{de::*, float::*, integer::*, ser::*};
use super::{Config, Result};

#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct OwnedValue(Value<'static>);

impl OwnedValue {
	#[must_use]
	pub fn new(value: Value<'_>) -> Self {
		value.into_owned()
	}

	#[must_use]
	pub fn into_inner(self) -> Value<'static> {
		self.0
	}
}

impl Deref for OwnedValue {
	type Target = Value<'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for OwnedValue {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'de> Deserialize<'de> for OwnedValue {
	fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer).map(Value::into_owned)
	}
}

impl Serialize for OwnedValue {
	fn serialize<S: Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
		self.0.serialize(serializer)
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Iter<T>(Box<dyn AllIteratorTrait<T> + Send + Sync>);

impl<T> Iter<T> {
	fn new<I>(iter: I) -> Self
	where
		I: AllIteratorTrait<T> + Send + Sync + 'static,
	{
		Self(Box::new(iter))
	}
}

impl<T> DoubleEndedIterator for Iter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back()
	}
}

impl<T> ExactSizeIterator for Iter<T> {
	fn len(&self) -> usize {
		self.0.len()
	}
}

impl<T> Iterator for Iter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	fn count(self) -> usize {
		self.len()
	}

	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
struct ValueVisitor<'a>(PhantomData<Value<'a>>);

impl<'de> Visitor<'de> for ValueVisitor<'de> {
	type Value = Value<'de>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("any value")
	}

	fn visit_bool<E>(self, v: bool) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Bool(v))
	}

	fn visit_i64<E>(self, v: i64) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_i128(v.into())
	}

	fn visit_i128<E>(self, v: i128) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(v.into())
	}

	fn visit_u64<E>(self, v: u64) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_u128(v.into())
	}

	fn visit_u128<E>(self, v: u128) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(v.into())
	}

	fn visit_f32<E>(self, v: f32) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(v.into())
	}

	fn visit_f64<E>(self, v: f64) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(v.into())
	}

	fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::String(Cow::Owned(v.to_owned())))
	}

	fn visit_borrowed_str<E>(self, v: &'de str) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::String(Cow::Borrowed(v)))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Bytes(Cow::Owned(v.to_owned())))
	}

	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Bytes(Cow::Borrowed(v)))
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Bytes(Cow::Owned(v)))
	}

	fn visit_none<E>(self) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Null)
	}

	fn visit_some<D>(self, deserializer: D) -> core::result::Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(self)
	}

	fn visit_unit<E>(self) -> core::result::Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Null)
	}

	fn visit_seq<A>(self, mut seq: A) -> core::result::Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let mut array = seq
			.size_hint()
			.map_or_else(VecDeque::new, VecDeque::with_capacity);

		while let Some(value) = seq.next_element()? {
			array.push_back(value);
		}

		Ok(Value::Array(array))
	}

	fn visit_map<A>(self, mut map: A) -> core::result::Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut entries = map
			.size_hint()
			.map_or_else(VecDeque::new, VecDeque::with_capacity);

		while let Some((key, value)) = map.next_entry()? {
			entries.push_back((key, value));
		}

		Ok(Value::Map(entries))
	}
}

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

	#[must_use]
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Null => true,
			Self::Bytes(b) => b.is_empty(),
			Self::String(s) => s.is_empty(),
			Self::Array(a) => a.is_empty(),
			Self::Map(m) => m.is_empty(),
			_ => false,
		}
	}

	#[must_use]
	pub const fn as_bool(&self) -> Option<bool> {
		if let Self::Bool(v) = self {
			Some(*v)
		} else {
			None
		}
	}

	#[must_use]
	pub const fn as_int(&self) -> Option<Integer> {
		if let Self::Integer(v) = self {
			Some(*v)
		} else {
			None
		}
	}

	#[must_use]
	pub const fn as_float(&self) -> Option<Float> {
		if let Self::Float(v) = self {
			Some(*v)
		} else {
			None
		}
	}

	#[must_use]
	pub fn as_bytes(&self) -> Option<&[u8]> {
		if let Self::Bytes(v) = self {
			Some(v)
		} else {
			None
		}
	}

	#[must_use]
	pub fn as_string(&self) -> Option<&str> {
		if let Self::String(s) = self {
			Some(s)
		} else {
			None
		}
	}

	#[must_use]
	pub const fn as_array(&self) -> Option<&VecDeque<Self>> {
		if let Self::Array(a) = self {
			Some(a)
		} else {
			None
		}
	}

	#[must_use]
	pub const fn as_map(&self) -> Option<&VecDeque<(Self, Self)>> {
		if let Self::Map(m) = self {
			Some(m)
		} else {
			None
		}
	}

	#[must_use]
	pub fn into_values(self) -> Iter<Value<'static>> {
		match self.into_owned().0 {
			Value::Array(a) => Iter::new(a.into_iter()),
			Value::Map(m) => Iter::new(m.into_iter().map(|(_, value)| value)),
			_ => Iter::new(core::iter::empty()),
		}
	}
}

impl<'de> Deserialize<'de> for Value<'de> {
	fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_any(ValueVisitor::default())
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

impl From<u8> for Value<'_> {
	fn from(value: u8) -> Self {
		Self::Integer(value.into())
	}
}

impl From<u16> for Value<'_> {
	fn from(value: u16) -> Self {
		Self::Integer(value.into())
	}
}

impl From<u32> for Value<'_> {
	fn from(value: u32) -> Self {
		Self::Integer(value.into())
	}
}

impl From<u64> for Value<'_> {
	fn from(value: u64) -> Self {
		Self::Integer(value.into())
	}
}

impl From<u128> for Value<'_> {
	fn from(value: u128) -> Self {
		Self::Integer(value.into())
	}
}

impl From<i8> for Value<'_> {
	fn from(value: i8) -> Self {
		Self::Integer(value.into())
	}
}

impl From<i16> for Value<'_> {
	fn from(value: i16) -> Self {
		Self::Integer(value.into())
	}
}

impl From<i32> for Value<'_> {
	fn from(value: i32) -> Self {
		Self::Integer(value.into())
	}
}

impl From<i64> for Value<'_> {
	fn from(value: i64) -> Self {
		Self::Integer(value.into())
	}
}

impl From<i128> for Value<'_> {
	fn from(value: i128) -> Self {
		Self::Integer(value.into())
	}
}

impl From<f32> for Value<'_> {
	fn from(value: f32) -> Self {
		Self::Float(value.into())
	}
}

impl From<f64> for Value<'_> {
	fn from(value: f64) -> Self {
		Self::Float(value.into())
	}
}

impl From<Option<Self>> for Value<'_> {
	fn from(value: Option<Self>) -> Self {
		value.unwrap_or_default()
	}
}

impl From<()> for Value<'_> {
	fn from((): ()) -> Self {
		Value::Null
	}
}

impl From<bool> for Value<'_> {
	fn from(value: bool) -> Self {
		Self::Bool(value)
	}
}

impl<'a> From<&'a [u8]> for Value<'a> {
	fn from(value: &'a [u8]) -> Self {
		Self::Bytes(Cow::Borrowed(value))
	}
}

impl From<Vec<u8>> for Value<'_> {
	fn from(value: Vec<u8>) -> Self {
		Self::Bytes(Cow::Owned(value))
	}
}

impl<'a> From<Cow<'a, [u8]>> for Value<'a> {
	fn from(value: Cow<'a, [u8]>) -> Self {
		Self::Bytes(value)
	}
}

impl<'a> From<&'a str> for Value<'a> {
	fn from(value: &'a str) -> Self {
		Self::String(Cow::Borrowed(value))
	}
}

impl From<String> for Value<'_> {
	fn from(value: String) -> Self {
		Self::String(Cow::Owned(value))
	}
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
	fn from(value: Cow<'a, str>) -> Self {
		Self::String(value)
	}
}

impl From<VecDeque<Self>> for Value<'_> {
	fn from(value: VecDeque<Self>) -> Self {
		Self::Array(value)
	}
}

impl From<VecDeque<(Self, Self)>> for Value<'_> {
	fn from(value: VecDeque<(Self, Self)>) -> Self {
		Self::Map(value)
	}
}

impl<T> FromIterator<T> for Value<'_>
where
	T: Into<Self>,
{
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = T>,
	{
		Self::from(iter.into_iter().map(Into::into).collect::<VecDeque<_>>())
	}
}

impl<K, V> FromIterator<(K, V)> for Value<'_>
where
	K: Into<Self>,
	V: Into<Self>,
{
	fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
		Self::from(
			iter.into_iter()
				.map(|(k, v)| (k.into(), v.into()))
				.collect::<VecDeque<_>>(),
		)
	}
}

impl PartialEq for Value<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Null, Self::Null) => true,
			(Self::Bool(lhs), Self::Bool(rhs)) => PartialEq::eq(lhs, rhs),
			(Self::Integer(lhs), Self::Integer(rhs)) => PartialEq::eq(lhs, rhs),
			(Self::Float(lhs), Self::Float(rhs)) => PartialEq::eq(lhs, rhs),
			(Self::Bytes(lhs), Self::Bytes(rhs)) => PartialEq::eq(lhs, rhs),
			(Self::String(lhs), Self::String(rhs)) => PartialEq::eq(lhs, rhs),
			(Self::Array(lhs), Self::Array(rhs)) => PartialEq::eq(lhs, rhs),
			(Self::Map(lhs), Self::Map(rhs)) => PartialEq::eq(lhs, rhs),
			_ => false,
		}
	}
}

impl PartialEq<bool> for Value<'_> {
	fn eq(&self, other: &bool) -> bool {
		match self {
			Self::Bool(b) => PartialEq::eq(b, other),
			_ => false,
		}
	}
}

impl PartialEq<u8> for Value<'_> {
	fn eq(&self, other: &u8) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<u16> for Value<'_> {
	fn eq(&self, other: &u16) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<u32> for Value<'_> {
	fn eq(&self, other: &u32) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<u64> for Value<'_> {
	fn eq(&self, other: &u64) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<u128> for Value<'_> {
	fn eq(&self, other: &u128) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<i8> for Value<'_> {
	fn eq(&self, other: &i8) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<i16> for Value<'_> {
	fn eq(&self, other: &i16) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<i32> for Value<'_> {
	fn eq(&self, other: &i32) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<i64> for Value<'_> {
	fn eq(&self, other: &i64) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<i128> for Value<'_> {
	fn eq(&self, other: &i128) -> bool {
		match self {
			Self::Integer(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<f32> for Value<'_> {
	fn eq(&self, other: &f32) -> bool {
		match self {
			Self::Float(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<f64> for Value<'_> {
	fn eq(&self, other: &f64) -> bool {
		match self {
			Self::Float(lhs) => PartialEq::eq(lhs, other),
			_ => false,
		}
	}
}

impl PartialEq<[u8]> for Value<'_> {
	fn eq(&self, other: &[u8]) -> bool {
		match self {
			Self::Bytes(bytes) => PartialEq::eq(bytes.as_ref(), other),
			_ => false,
		}
	}
}

impl PartialEq<str> for Value<'_> {
	fn eq(&self, other: &str) -> bool {
		match self {
			Self::String(s) => PartialEq::eq(s, other),
			_ => false,
		}
	}
}

impl Serialize for Value<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
		match self {
			Self::Null => serializer.serialize_none(),
			Self::Bool(b) => serializer.serialize_bool(*b),
			Self::Integer(Integer::Unsigned(int)) => serializer.serialize_u128(*int),
			Self::Integer(Integer::Signed(int)) => serializer.serialize_i128(*int),
			Self::Float(Float::F32(f)) => serializer.serialize_f32(*f),
			Self::Float(Float::F64(f)) => serializer.serialize_f64(*f),
			Self::Bytes(bytes) => serializer.serialize_bytes(bytes),
			Self::String(s) => serializer.serialize_str(s),
			Self::Array(arr) => {
				let mut seq = serializer.serialize_seq(Some(arr.len()))?;

				for value in arr {
					seq.serialize_element(value)?;
				}

				seq.end()
			}
			Self::Map(map) => {
				let mut seq = serializer.serialize_map(Some(map.len()))?;

				for (key, value) in map {
					seq.serialize_entry(key, value)?;
				}

				seq.end()
			}
		}
	}
}

trait AllIteratorTrait<Item>:
	Iterator<Item = Item> + ExactSizeIterator + DoubleEndedIterator + Debug
{
}

impl<I, T> AllIteratorTrait<T> for I where
	I: Iterator<Item = T> + ExactSizeIterator + DoubleEndedIterator + Debug
{
}

pub fn from_value_with_config<'de, T>(value: Value<'de>, _: Config) -> Result<T>
where
	T: Deserialize<'de>,
{
	let de = ValueDeserializer::new(value);
	T::deserialize(de)
}

pub fn from_value<'de, T>(value: Value<'de>) -> Result<T>
where
	T: Deserialize<'de>,
{
	from_value_with_config(value, Config::default())
}

pub fn to_value_with_config<T: Serialize>(value: &T, config: Config) -> Result<Value<'static>> {
	let ser = ValueSerializer::new(config.use_indices);
	value.serialize(ser)
}

pub fn to_value<T: Serialize>(value: &T) -> Result<Value<'static>> {
	to_value_with_config(value, Config::default())
}
