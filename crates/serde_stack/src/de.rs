use std::fmt::{Formatter, Result as FmtResult};

use serde::de::{
	DeserializeSeed as SerdeDeserializeSeed, Deserializer as SerdeDeserializer,
	EnumAccess as SerdeEnumAccess, Error as DeError, MapAccess as SerdeMapAccess,
	SeqAccess as SerdeSeqAccess, VariantAccess as SerdeVariantAccess, Visitor as SerdeVisitor,
};

use super::param::Param;

pub struct Deserializer<D> {
	pub inner: D,
	pub red_zone: usize,
	pub stack_size: usize,
}

impl<D> Deserializer<D> {
	pub fn new(inner: D) -> Self {
		let default_params = Param::default();
		Self {
			inner,
			stack_size: default_params.stack_size,
			red_zone: default_params.red_zone,
		}
	}

	const fn param(&self) -> Param {
		Param::new(self.red_zone, self.stack_size)
	}
}

impl<'de, D> SerdeDeserializer<'de> for Deserializer<D>
where
	D: SerdeDeserializer<'de>,
{
	type Error = D::Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_any(Visitor::new(visitor, param))
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_bool(Visitor::new(visitor, param))
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_i8(Visitor::new(visitor, param))
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_i16(Visitor::new(visitor, param))
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_i32(Visitor::new(visitor, param))
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_i64(Visitor::new(visitor, param))
	}

	fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_i128(Visitor::new(visitor, param))
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_u8(Visitor::new(visitor, param))
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_u16(Visitor::new(visitor, param))
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_u32(Visitor::new(visitor, param))
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_u64(Visitor::new(visitor, param))
	}

	fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_u128(Visitor::new(visitor, param))
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_f32(Visitor::new(visitor, param))
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_f64(Visitor::new(visitor, param))
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_char(Visitor::new(visitor, param))
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_str(Visitor::new(visitor, param))
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_string(Visitor::new(visitor, param))
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_bytes(Visitor::new(visitor, param))
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_byte_buf(Visitor::new(visitor, param))
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_option(Visitor::new(visitor, param))
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_unit(Visitor::new(visitor, param))
	}

	fn deserialize_unit_struct<V>(
		self,
		name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_unit_struct(name, Visitor::new(visitor, param))
	}

	fn deserialize_newtype_struct<V>(
		self,
		name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_newtype_struct(name, Visitor::new(visitor, param))
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_seq(Visitor::new(visitor, param))
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_tuple(len, Visitor::new(visitor, param))
	}

	fn deserialize_tuple_struct<V>(
		self,
		name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_tuple_struct(name, len, Visitor::new(visitor, param))
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner.deserialize_map(Visitor::new(visitor, param))
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_struct(name, fields, Visitor::new(visitor, param))
	}

	fn deserialize_enum<V>(
		self,
		name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_enum(name, variants, Visitor::new(visitor, param))
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_identifier(Visitor::new(visitor, param))
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		let param = self.param();
		self.inner
			.deserialize_ignored_any(Visitor::new(visitor, param))
	}

	fn is_human_readable(&self) -> bool {
		self.inner.is_human_readable()
	}
}

struct Visitor<V> {
	inner: V,
	param: Param,
}

impl<V> Visitor<V> {
	pub const fn new(inner: V, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, V> SerdeVisitor<'de> for Visitor<V>
where
	V: SerdeVisitor<'de>,
{
	type Value = V::Value;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		self.inner.expecting(formatter)
	}

	fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_bool(v)
	}

	fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i8(v)
	}

	fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i16(v)
	}

	fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i32(v)
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i64(v)
	}

	fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i128(v)
	}

	fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u8(v)
	}

	fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u16(v)
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u32(v)
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u64(v)
	}

	fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u128(v)
	}

	fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_f32(v)
	}

	fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_f64(v)
	}

	fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_char(v)
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_str(v)
	}

	fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_borrowed_str(v)
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_string(v)
	}

	fn visit_unit<E>(self) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_unit()
	}

	fn visit_none<E>(self) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_none()
	}

	fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: SerdeDeserializer<'de>,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			self.inner.visit_some(Deserializer {
				inner: deserializer,
				red_zone: self.param.red_zone,
				stack_size: self.param.stack_size,
			})
		})
	}

	fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: SerdeDeserializer<'de>,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			self.inner.visit_newtype_struct(Deserializer {
				inner: deserializer,
				red_zone: self.param.red_zone,
				stack_size: self.param.stack_size,
			})
		})
	}

	fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
	where
		A: SerdeSeqAccess<'de>,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			self.inner.visit_seq(SeqAccess::new(seq, self.param))
		})
	}

	fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
	where
		A: SerdeMapAccess<'de>,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			self.inner.visit_map(MapAccess::new(map, self.param))
		})
	}

	fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
	where
		A: SerdeEnumAccess<'de>,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			self.inner.visit_enum(EnumAccess::new(data, self.param))
		})
	}

	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_borrowed_bytes(v)
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_byte_buf(v)
	}
}

struct MapAccess<D> {
	inner: D,
	param: Param,
}

impl<D> MapAccess<D> {
	const fn new(inner: D, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, D> SerdeMapAccess<'de> for MapAccess<D>
where
	D: SerdeMapAccess<'de>,
{
	type Error = D::Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: SerdeDeserializeSeed<'de>,
	{
		self.inner
			.next_key_seed(DeserializeSeed::new(seed, self.param))
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeDeserializeSeed<'de>,
	{
		self.inner
			.next_value_seed(DeserializeSeed::new(seed, self.param))
	}

	fn size_hint(&self) -> Option<usize> {
		self.inner.size_hint()
	}
}

struct DeserializeSeed<S> {
	inner: S,
	param: Param,
}

impl<S> DeserializeSeed<S> {
	const fn new(inner: S, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, S> SerdeDeserializeSeed<'de> for DeserializeSeed<S>
where
	S: SerdeDeserializeSeed<'de>,
{
	type Value = S::Value;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: SerdeDeserializer<'de>,
	{
		self.inner.deserialize(Deserializer {
			inner: deserializer,
			red_zone: self.param.red_zone,
			stack_size: self.param.stack_size,
		})
	}
}

struct SeqAccess<D> {
	inner: D,
	param: Param,
}

impl<D> SeqAccess<D> {
	const fn new(inner: D, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, D> SerdeSeqAccess<'de> for SeqAccess<D>
where
	D: SerdeSeqAccess<'de>,
{
	type Error = D::Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: SerdeDeserializeSeed<'de>,
	{
		self.inner
			.next_element_seed(DeserializeSeed::new(seed, self.param))
	}

	fn size_hint(&self) -> Option<usize> {
		self.inner.size_hint()
	}
}

struct VariantAccess<D> {
	inner: D,
	param: Param,
}

impl<D> VariantAccess<D> {
	const fn new(inner: D, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, D> SerdeVariantAccess<'de> for VariantAccess<D>
where
	D: SerdeVariantAccess<'de>,
{
	type Error = D::Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		self.inner.unit_variant()
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: SerdeDeserializeSeed<'de>,
	{
		self.inner
			.newtype_variant_seed(DeserializeSeed::new(seed, self.param))
	}

	fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		self.inner
			.tuple_variant(len, Visitor::new(visitor, self.param))
	}

	fn struct_variant<V>(
		self,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: SerdeVisitor<'de>,
	{
		self.inner
			.struct_variant(fields, Visitor::new(visitor, self.param))
	}
}

struct EnumAccess<D> {
	inner: D,
	param: Param,
}

impl<D> EnumAccess<D> {
	const fn new(inner: D, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, D> SerdeEnumAccess<'de> for EnumAccess<D>
where
	D: SerdeEnumAccess<'de>,
{
	type Error = D::Error;
	type Variant = VariantAccess<D::Variant>;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: SerdeDeserializeSeed<'de>,
	{
		let param = self.param;
		self.inner
			.variant_seed(DeserializeSeed::new(seed, param))
			.map(|(v, vis)| (v, VariantAccess::new(vis, param)))
	}
}
