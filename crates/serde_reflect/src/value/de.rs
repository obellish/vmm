use serde::de::{
	DeserializeSeed, Deserializer, EnumAccess, Error as DeError, IntoDeserializer, MapAccess,
	SeqAccess, VariantAccess, Visitor,
};

use crate::{Error, Result, Value};

#[repr(transparent)]
pub struct ValueDeserializer<'de> {
	value: &'de Value,
}

impl<'de> ValueDeserializer<'de> {
	#[must_use]
	pub const fn new(value: &'de Value) -> Self {
		Self { value }
	}
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, _: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::NotSupported("deserialize_any"))
	}

	fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Bool(b) => visitor.visit_bool(*b),
			other => invalid_type::<V>(other, "bool"),
		}
	}

	fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_i128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_u128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_integer(self.value, visitor)
	}

	fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_float(self.value, visitor)
	}

	fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visit_float(self.value, visitor)
	}

	fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Char(c) => visitor.visit_char(*c),
			Value::Str(s) if matches!(s.len(), 1) => visitor.visit_char(s.chars().next().unwrap()),
			other => invalid_type::<V>(other, "char"),
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Str(s) => visitor.visit_borrowed_str(s),
			other => invalid_type::<V>(other, "str"),
		}
	}

	fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Str(s) => visitor.visit_string(s.clone()),
			other => invalid_type::<V>(other, "string"),
		}
	}

	fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Bytes(b) => visitor.visit_borrowed_bytes(b),
			Value::Str(s) => visitor.visit_bytes(s.as_bytes()),
			other => invalid_type::<V>(other, "bytes"),
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Bytes(b) => visitor.visit_byte_buf(b.clone()),
			Value::Str(s) => visitor.visit_bytes(s.as_bytes()),
			other => invalid_type::<V>(other, "byte_buf"),
		}
	}

	fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Option(None) => visitor.visit_none(),
			Value::Option(Some(v)) => visitor.visit_some(v.into_deserializer()),
			other => invalid_type::<V>(other, "option"),
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Unit => visitor.visit_unit(),
			other => invalid_type::<V>(other, "unit"),
		}
	}

	fn deserialize_unit_struct<V>(
		self,
		_: &'static str,
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Unit => visitor.visit_unit(),
			other => invalid_type::<V>(other, "unit struct"),
		}
	}

	fn deserialize_newtype_struct<V>(
		self,
		_: &'static str,
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
			other => invalid_type::<V>(other, "sequence"),
		}
	}

	fn deserialize_tuple<V>(
		self,
		_: usize,
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
			other => invalid_type::<V>(other, "tuple"),
		}
	}

	fn deserialize_tuple_struct<V>(
		self,
		_: &'static str,
		_: usize,
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
			other => invalid_type::<V>(other, "tuple struct"),
		}
	}

	fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Seq(s) => visitor.visit_seq(s.into_seq_deserializer()),
			other => invalid_type::<V>(other, "map"),
		}
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_tuple_struct(name, fields.len(), visitor)
	}

	fn deserialize_enum<V>(
		self,
		_: &'static str,
		_: &'static [&'static str],
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Variant(index, variant) => {
				let inner = ValueEnumAccess {
					index: *index,
					value: variant,
				};
				visitor.visit_enum(inner)
			}
			other => invalid_type::<V>(other, "enum variant"),
		}
	}

	fn deserialize_identifier<V>(self, _: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::NotSupported("deserialize_identifier"))
	}

	fn deserialize_ignored_any<V>(self, _: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::NotSupported("deserialize_ignored_any"))
	}
}

#[repr(transparent)]
pub struct ValueAccess<I> {
	values: I,
}

impl<I> ValueAccess<I> {
	const fn new(values: I) -> Self {
		Self { values }
	}
}

impl<'de, I> MapAccess<'de> for ValueAccess<I>
where
	I: Iterator<Item = &'de Value>,
{
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
	where
		K: DeserializeSeed<'de>,
	{
		match self.values.next() {
			Some(x) => seed.deserialize(x.into_deserializer()).map(Some),
			None => Ok(None),
		}
	}

	fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		match self.values.next() {
			Some(x) => seed.deserialize(x.into_deserializer()),
			None => Err(Error::Deserialization("value in map")),
		}
	}

	fn size_hint(&self) -> Option<usize> {
		self.values.size_hint().1.map(|x| x / 2)
	}
}

impl<'de, I> SeqAccess<'de> for ValueAccess<I>
where
	I: Iterator<Item = &'de Value>,
{
	type Error = Error;

	fn next_element_seed<T>(
		&mut self,
		seed: T,
	) -> std::result::Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		match self.values.next() {
			Some(x) => seed.deserialize(x.into_deserializer()).map(Some),
			None => Ok(None),
		}
	}

	fn size_hint(&self) -> Option<usize> {
		self.values.size_hint().1
	}
}

pub struct ValueEnumAccess<'de> {
	index: u32,
	value: &'de Value,
}

impl<'de> EnumAccess<'de> for ValueEnumAccess<'de> {
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> std::result::Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let value = seed.deserialize(self.index.into_deserializer())?;
		Ok((value, self))
	}
}

impl<'de> VariantAccess<'de> for ValueEnumAccess<'de> {
	type Error = Error;

	fn unit_variant(self) -> std::result::Result<(), Self::Error> {
		match self.value {
			Value::Unit => Ok(()),
			other => Err(DeError::invalid_type(other.into(), &"unit")),
		}
	}

	fn newtype_variant_seed<T>(self, seed: T) -> std::result::Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(self.value.into_deserializer())
	}

	fn tuple_variant<V>(self, _: usize, visitor: V) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
			other => Err(DeError::invalid_type(other.into(), &"tuple variant")),
		}
	}

	fn struct_variant<V>(
		self,
		_: &'static [&'static str],
		visitor: V,
	) -> std::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		match self.value {
			Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
			other => Err(DeError::invalid_type(other.into(), &"struct variant")),
		}
	}
}

pub(crate) trait IntoSeqDeserializer {
	type SeqDeserializer;

	fn into_seq_deserializer(self) -> Self::SeqDeserializer;
}

impl<'de> IntoSeqDeserializer for &'de [Value] {
	type SeqDeserializer = ValueAccess<std::slice::Iter<'de, Value>>;

	fn into_seq_deserializer(self) -> Self::SeqDeserializer {
		ValueAccess::new(self.iter())
	}
}

fn visit_integer<'de, V>(v: &Value, visitor: V) -> Result<V::Value>
where
	V: Visitor<'de>,
{
	match v {
		Value::I8(v) => visitor.visit_i8(*v),
		Value::I16(v) => visitor.visit_i16(*v),
		Value::I32(v) => visitor.visit_i32(*v),
		Value::I64(v) => visitor.visit_i64(*v),
		Value::I128(v) => visitor.visit_i128(*v),
		Value::U8(v) => visitor.visit_u8(*v),
		Value::U16(v) => visitor.visit_u16(*v),
		Value::U32(v) => visitor.visit_u32(*v),
		Value::U64(v) => visitor.visit_u64(*v),
		Value::U128(v) => visitor.visit_u128(*v),
		other => invalid_type::<V>(other, "int"),
	}
}

fn visit_float<'de, V>(value: &Value, visitor: V) -> Result<V::Value>
where
	V: Visitor<'de>,
{
	match value {
		Value::F32(f) => visitor.visit_f32(*f),
		Value::F64(f) => visitor.visit_f64(*f),
		other => invalid_type::<V>(other, "float"),
	}
}

fn invalid_type<'de, V>(value: &Value, expected: &str) -> Result<V::Value>
where
	V: Visitor<'de>,
{
	Err(DeError::invalid_type(value.into(), &expected))
}
