use alloc::{borrow::Cow, collections::VecDeque};
use core::mem;

use serde::de::{
	DeserializeSeed, Deserializer, EnumAccess, Error as DeError, IntoDeserializer, MapAccess,
	SeqAccess, Unexpected, VariantAccess, Visitor,
};

use super::{Float, Integer, Value};
use crate::{Error, Result};

#[repr(transparent)]
pub struct ValueDeserializer<'a>(Value<'a>);

impl<'de> ValueDeserializer<'de> {
	#[must_use]
	pub const fn new(value: Value<'de>) -> Self {
		Self(value)
	}

	fn deserialize<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Null => visitor.visit_none(),
			Value::Bool(b) => visitor.visit_bool(b),
			Value::Integer(int) => visit_integer(int, visitor),
			Value::Float(Float::F32(float)) => visitor.visit_f32(float),
			Value::Float(Float::F64(float)) => visitor.visit_f64(float),
			Value::Bytes(Cow::Borrowed(bytes)) => visitor.visit_borrowed_bytes(bytes),
			Value::Bytes(Cow::Owned(bytes)) => visitor.visit_byte_buf(bytes),
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			Value::Array(arr) => visitor.visit_seq(ValueSeqAccess(arr)),
			Value::Map(map) => visitor.visit_map(ValueMapAccess(map)),
		}
	}
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Self::deserialize(self, visitor)
	}

	fn deserialize_bool<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Bool(v) => visitor.visit_bool(v),
			other => invalid_type::<V>(other, "bool"),
		}
	}

	fn deserialize_i8<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "i8"),
		}
	}

	fn deserialize_i16<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "i16"),
		}
	}

	fn deserialize_i32<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "i32"),
		}
	}

	fn deserialize_i64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "i64"),
		}
	}

	fn deserialize_i128<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "i128"),
		}
	}

	fn deserialize_u8<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "u8"),
		}
	}

	fn deserialize_u16<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "u16"),
		}
	}

	fn deserialize_u32<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "u32"),
		}
	}

	fn deserialize_u64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "u64"),
		}
	}

	fn deserialize_u128<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(int) => visit_integer(int, visitor),
			other => invalid_type::<V>(other, "u128"),
		}
	}

	fn deserialize_f32<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Float(Float::F32(f)) => visitor.visit_f32(f),
			Value::Float(Float::F64(f)) => visitor.visit_f64(f),
			other => invalid_type::<V>(other, "float"),
		}
	}

	fn deserialize_f64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Float(Float::F32(f)) => visitor.visit_f32(f),
			Value::Float(Float::F64(f)) => visitor.visit_f64(f),
			other => invalid_type::<V>(other, "float"),
		}
	}

	fn deserialize_char<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			other => invalid_type::<V>(other, "char"),
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
			Value::String(Cow::Owned(s)) => visitor.visit_string(s),
			other => invalid_type::<V>(other, "string"),
		}
	}

	fn deserialize_string<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
			Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
			other => invalid_type::<V>(other, "bytes"),
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
			Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
			other => invalid_type::<V>(other, "bytes"),
		}
	}

	fn deserialize_option<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		if matches!(self.0, Value::Null) {
			visitor.visit_none()
		} else {
			visitor.visit_some(self)
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Null => visitor.visit_unit(),
			other => invalid_type::<V>(other, "unit"),
		}
	}

	fn deserialize_unit_struct<V>(
		self,
		_: &'static str,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_newtype_struct<V>(
		self,
		_: &'static str,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Array(arr) => visitor.visit_seq(ValueSeqAccess(arr)),
			other => invalid_type::<V>(other, "sequence"),
		}
	}

	fn deserialize_tuple<V>(
		self,
		_: usize,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Array(arr) => visitor.visit_seq(ValueSeqAccess(arr)),
			other => invalid_type::<V>(other, "tuple"),
		}
	}

	fn deserialize_tuple_struct<V>(
		self,
		_: &'static str,
		_: usize,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Array(arr) => visitor.visit_seq(ValueSeqAccess(arr)),
			other => invalid_type::<V>(other, "tuple struct"),
		}
	}

	fn deserialize_map<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Map(map) => visitor.visit_map(ValueMapAccess(map)),
			other => invalid_type::<V>(other, "map"),
		}
	}

	fn deserialize_struct<V>(
		self,
		_: &'static str,
		_: &'static [&'static str],
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_enum<V>(
		self,
		_: &'static str,
		_: &'static [&'static str],
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(Integer::Unsigned(int)) => {
				visitor.visit_enum((int as u32).into_deserializer())
			}
			Value::String(s) => visitor.visit_enum(s.as_ref().into_deserializer()),
			Value::Map(map) => visitor.visit_enum(ValueEnumAccess(map)),
			other => invalid_type::<V>(other, "enum"),
		}
	}

	fn deserialize_identifier<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.0 {
			Value::Integer(Integer::Unsigned(_)) => self.deserialize_u32(visitor),
			Value::String(..) => self.deserialize_str(visitor),
			other => invalid_type::<V>(other, "identifier"),
		}
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}

	fn is_human_readable(&self) -> bool {
		true
	}
}

impl<'de> VariantAccess<'de> for ValueDeserializer<'de> {
	type Error = Error;

	fn unit_variant(self) -> core::result::Result<(), Self::Error> {
		Err(Error::invalid_type(
			Unexpected::from(&self.0),
			&"unit variant",
		))
	}

	fn newtype_variant_seed<T>(self, seed: T) -> core::result::Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(self)
	}

	fn tuple_variant<V>(self, _: usize, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Deserializer::deserialize_seq(self, visitor)
	}

	fn struct_variant<V>(
		self,
		_: &'static [&'static str],
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Deserializer::deserialize_map(self, visitor)
	}
}

#[repr(transparent)]
struct ValueEnumAccess<'de>(VecDeque<(Value<'de>, Value<'de>)>);

impl<'de> EnumAccess<'de> for ValueEnumAccess<'de> {
	type Error = Error;
	type Variant = ValueDeserializer<'de>;

	fn variant_seed<V>(
		mut self,
		seed: V,
	) -> core::result::Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		if !matches!(self.0.len(), 1) {
			return Err(DeError::invalid_length(1, &"exactly one key-value pair"));
		}

		let (key, value) = self.0.pop_front().unwrap();
		let res = seed.deserialize(ValueDeserializer(key))?;
		Ok((res, ValueDeserializer(value)))
	}
}

#[repr(transparent)]
struct ValueSeqAccess<'de>(VecDeque<Value<'de>>);

impl<'de> SeqAccess<'de> for ValueSeqAccess<'de> {
	type Error = Error;

	fn next_element_seed<T>(
		&mut self,
		seed: T,
	) -> core::result::Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		if let Some(value) = self.0.pop_front() {
			seed.deserialize(ValueDeserializer(value)).map(Some)
		} else {
			Ok(None)
		}
	}

	fn size_hint(&self) -> Option<usize> {
		Some(self.0.len())
	}
}

#[repr(transparent)]
struct ValueMapAccess<'de>(VecDeque<(Value<'de>, Value<'de>)>);

impl<'de> MapAccess<'de> for ValueMapAccess<'de> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> core::result::Result<Option<K::Value>, Self::Error>
	where
		K: DeserializeSeed<'de>,
	{
		if let Some((key, _)) = self.0.front_mut() {
			let value = mem::replace(key, Value::Null);
			Ok(Some(seed.deserialize(ValueDeserializer(value))?))
		} else {
			Ok(None)
		}
	}

	fn next_value_seed<V>(&mut self, seed: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		if let Some((_, value)) = self.0.pop_front() {
			Ok(seed.deserialize(ValueDeserializer(value))?)
		} else {
			Err(DeError::custom(
				"next_value_seed called without next_key_seed",
			))
		}
	}

	fn next_entry_seed<K, V>(
		&mut self,
		kseed: K,
		vseed: V,
	) -> core::result::Result<Option<(K::Value, V::Value)>, Self::Error>
	where
		K: DeserializeSeed<'de>,
		V: DeserializeSeed<'de>,
	{
		if let Some((key, value)) = self.0.pop_front() {
			let key = kseed.deserialize(ValueDeserializer(key))?;
			let value = vseed.deserialize(ValueDeserializer(value))?;
			Ok(Some((key, value)))
		} else {
			Ok(None)
		}
	}

	fn size_hint(&self) -> Option<usize> {
		Some(self.0.len())
	}
}

fn visit_integer<'de, V>(int: Integer, visitor: V) -> Result<V::Value>
where
	V: Visitor<'de>,
{
	match int {
		Integer::Unsigned(int) if int <= u128::from(u8::MAX) => visitor.visit_u8(int as u8),
		Integer::Unsigned(int) if int <= u128::from(u16::MAX) => visitor.visit_u16(int as u16),
		Integer::Unsigned(int) if int <= u128::from(u32::MAX) => visitor.visit_u32(int as u32),
		Integer::Unsigned(int) if int <= u128::from(u64::MAX) => visitor.visit_u64(int as u64),
		Integer::Unsigned(int) => visitor.visit_u128(int),
		Integer::Signed(int) if (i128::from(i8::MIN)..=i128::from(i8::MAX)).contains(&int) => {
			visitor.visit_i8(int as i8)
		}
		Integer::Signed(int) if (i128::from(i16::MIN)..=i128::from(i16::MAX)).contains(&int) => {
			visitor.visit_i16(int as i16)
		}
		Integer::Signed(int) if (i128::from(i32::MIN)..=i128::from(i32::MAX)).contains(&int) => {
			visitor.visit_i32(int as i32)
		}
		Integer::Signed(int) if (i128::from(i64::MIN)..=i128::from(i32::MAX)).contains(&int) => {
			visitor.visit_i64(int as i64)
		}
		Integer::Signed(int) => visitor.visit_i128(int),
	}
}

fn invalid_type<'de, V>(value: Value<'de>, expected: &str) -> Result<V::Value>
where
	V: Visitor<'de>,
{
	Err(DeError::invalid_type(Unexpected::from(&value), &expected))
}
