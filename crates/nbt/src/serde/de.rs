use std::{
	fmt::{Formatter, Result as FmtResult},
	hash::Hash,
	marker::PhantomData,
};

use serde::{
	Deserialize, Deserializer,
	de::{
		self, Error as DeError, IntoDeserializer, SeqAccess, Visitor,
		value::{
			MapAccessDeserializer, MapDeserializer, SeqAccessDeserializer, StrDeserializer,
			StringDeserializer,
		},
	},
	forward_to_deserialize_any,
};

use crate::{
	Compound, Error, List, Value,
	conv::{i8_vec_into_u8_vec, u8_slice_as_i8_slice, u8_vec_into_i8_vec},
};

impl<'de, S> Deserialize<'de> for Value<S>
where
	S: Deserialize<'de> + Hash + Ord,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(ValueVisitor::<S>(PhantomData))
	}
}

impl<'de, S> Deserialize<'de> for List<S>
where
	S: Deserialize<'de> + Hash + Ord,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_seq(ListVisitor::<S>(PhantomData))
	}
}

impl<'de> Deserializer<'de> for Compound {
	type Error = Error;

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct newtype_struct seq tuple
		tuple_struct map struct enum identifier ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_map(MapDeserializer::new(self.into_iter()))
	}
}

impl IntoDeserializer<'_, Error> for Compound {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer {
		self
	}
}

impl<'de> Deserializer<'de> for Value {
	type Error = Error;

	forward_to_deserialize_any! {
		i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf unit unit_struct newtype_struct seq tuple
		tuple_struct map struct identifier ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self {
			Self::Byte(v) => visitor.visit_i8(v),
			Self::Short(v) => visitor.visit_i16(v),
			Self::Int(v) => visitor.visit_i32(v),
			Self::Long(v) => visitor.visit_i64(v),
			Self::Float(v) => visitor.visit_f32(v),
			Self::Double(v) => visitor.visit_f64(v),
			Self::ByteArray(v) => visitor.visit_byte_buf(i8_vec_into_u8_vec(v)),
			Self::String(v) => visitor.visit_string(v),
			Self::List(v) => v.deserialize_any(visitor),
			Self::Compound(v) => v.into_deserializer().deserialize_any(visitor),
			Self::IntArray(v) => v.into_deserializer().deserialize_any(visitor),
			Self::LongArray(v) => v.into_deserializer().deserialize_any(visitor),
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self {
			Self::Byte(b) => visitor.visit_bool(!matches!(b, 0)),
			_ => self.deserialize_any(visitor),
		}
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_some(self)
	}

	fn deserialize_enum<V>(
		self,
		_: &'static str,
		_: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self {
			Self::String(s) => visitor.visit_enum(s.into_deserializer()),
			other => other.deserialize_any(visitor),
		}
	}
}

impl IntoDeserializer<'_, Error> for Value {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer {
		self
	}
}

impl<'de> Deserializer<'de> for List {
	type Error = Error;

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct newtype_struct seq tuple
		tuple_struct map struct enum identifier ignored_any
	}

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self {
			Self::End => visitor.visit_seq(EndSeqAccess),
			Self::Byte(v) => visitor.visit_byte_buf(i8_vec_into_u8_vec(v)),
			Self::Short(v) => v.into_deserializer().deserialize_any(visitor),
			Self::Int(v) => v.into_deserializer().deserialize_any(visitor),
			Self::Long(v) => v.into_deserializer().deserialize_any(visitor),
			Self::Float(v) => v.into_deserializer().deserialize_any(visitor),
			Self::Double(v) => v.into_deserializer().deserialize_any(visitor),
			Self::ByteArray(v) => v.into_deserializer().deserialize_any(visitor),
			Self::String(v) => v.into_deserializer().deserialize_any(visitor),
			Self::List(v) => v.into_deserializer().deserialize_any(visitor),
			Self::Compound(v) => v.into_deserializer().deserialize_any(visitor),
			Self::IntArray(v) => v.into_deserializer().deserialize_any(visitor),
			Self::LongArray(v) => v.into_deserializer().deserialize_any(visitor),
		}
	}
}

impl IntoDeserializer<'_, Error> for List {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer {
		self
	}
}

#[repr(transparent)]
struct ValueVisitor<S>(PhantomData<S>);

impl<'de, S> Visitor<'de> for ValueVisitor<S>
where
	S: Deserialize<'de> + Hash + Ord,
{
	type Value = Value<S>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a valid NBT type")
	}

	fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Byte(v.into()))
	}

	fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Byte(v))
	}

	fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Short(v))
	}

	fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Int(v))
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Long(v))
	}

	fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Byte(v as i8))
	}

	fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Short(v as i16))
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Int(v as i32))
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Long(v as i64))
	}

	fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Float(v))
	}

	fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::Double(v))
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		S::deserialize(StrDeserializer::new(v)).map(Value::String)
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		S::deserialize(StringDeserializer::new(v)).map(Value::String)
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::ByteArray(u8_slice_as_i8_slice(v).to_owned()))
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(Value::ByteArray(u8_vec_into_i8_vec(v)))
	}

	fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		Ok(List::deserialize(SeqAccessDeserializer::new(seq))?.into())
	}

	fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
	where
		A: de::MapAccess<'de>,
	{
		Ok(Compound::deserialize(MapAccessDeserializer::new(map))?.into())
	}
}

#[repr(transparent)]
struct ListVisitor<S>(PhantomData<S>);

impl<'de, S> Visitor<'de> for ListVisitor<S>
where
	S: Deserialize<'de> + Hash + Ord,
{
	type Value = List<S>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a sequence or bytes")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		match seq.next_element::<Value<S>>()? {
			Some(v) => match v {
				Value::Byte(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::Short(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::Int(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::Long(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::Float(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::Double(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::ByteArray(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::String(v) => deserialize_seq_remainder(v, seq, List::String),
				Value::List(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::Compound(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::IntArray(v) => deserialize_seq_remainder(v, seq, From::from),
				Value::LongArray(v) => deserialize_seq_remainder(v, seq, From::from),
			},
			None => Ok(List::End),
		}
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(List::Byte(u8_vec_into_i8_vec(v)))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(List::Byte(u8_slice_as_i8_slice(v).to_owned()))
	}
}

struct EndSeqAccess;

impl<'de> SeqAccess<'de> for EndSeqAccess {
	type Error = Error;

	fn next_element_seed<T>(&mut self, _: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: de::DeserializeSeed<'de>,
	{
		Ok(None)
	}
}

fn deserialize_seq_remainder<'de, T, A, S>(
	first: T,
	mut seq: A,
	conv: impl FnOnce(Vec<T>) -> List<S>,
) -> Result<List<S>, A::Error>
where
	T: Deserialize<'de>,
	A: SeqAccess<'de>,
{
	let mut vec = match seq.size_hint() {
		Some(n) => Vec::with_capacity(n + 1),
		None => Vec::new(),
	};

	vec.push(first);

	while let Some(v) = seq.next_element()? {
		vec.push(v);
	}

	Ok(conv(vec))
}
