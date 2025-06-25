use std::fmt::Display;

use serde::ser::{
	Serialize as SerdeSerialize, SerializeMap, SerializeSeq, SerializeStruct,
	SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
	Serializer as SerdeSerializer,
};

use super::param::Param;

pub struct Serializer<S> {
	pub inner: S,
	pub red_zone: usize,
	pub stack_size: usize,
}

impl<S> Serializer<S> {
	pub fn new(inner: S) -> Self {
		let default_param = Param::default();
		Self {
			inner,
			red_zone: default_param.red_zone,
			stack_size: default_param.stack_size,
		}
	}

	const fn param(&self) -> Param {
		Param::new(self.red_zone, self.stack_size)
	}
}

impl<S: SerdeSerializer> SerdeSerializer for Serializer<S> {
	type Error = S::Error;
	type Ok = S::Ok;
	type SerializeMap = SerializeHelper<S::SerializeMap>;
	type SerializeSeq = SerializeHelper<S::SerializeSeq>;
	type SerializeStruct = SerializeHelper<S::SerializeStruct>;
	type SerializeStructVariant = SerializeHelper<S::SerializeStructVariant>;
	type SerializeTuple = SerializeHelper<S::SerializeTuple>;
	type SerializeTupleStruct = SerializeHelper<S::SerializeTupleStruct>;
	type SerializeTupleVariant = SerializeHelper<S::SerializeTupleVariant>;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_bool(v)
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_i8(v)
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_i16(v)
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_i32(v)
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_i64(v)
	}

	fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_i128(v)
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_u8(v)
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_u16(v)
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_u32(v)
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_u64(v)
	}

	fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_u128(v)
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_f32(v)
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_f64(v)
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_char(v)
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_str(v)
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_bytes(v)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_none()
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		let param = self.param();
		self.inner.serialize_some(&Serialize::new(value, param))
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_unit()
	}

	fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
		self.inner.serialize_unit_struct(name)
	}

	fn serialize_unit_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		self.inner
			.serialize_unit_variant(name, variant_index, variant)
	}

	fn serialize_newtype_struct<T>(
		self,
		name: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		let param = self.param();
		self.inner
			.serialize_newtype_struct(name, &Serialize::new(value, param))
	}

	fn serialize_newtype_variant<T>(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		let param = self.param();
		self.inner.serialize_newtype_variant(
			name,
			variant_index,
			variant,
			&Serialize::new(value, param),
		)
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_seq(len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_tuple(len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn serialize_tuple_struct(
		self,
		name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_tuple_struct(name, len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn serialize_tuple_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_tuple_variant(name, variant_index, variant, len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_map(len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn serialize_struct(
		self,
		name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_struct(name, len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn serialize_struct_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		let param = self.param();
		self.inner
			.serialize_struct_variant(name, variant_index, variant, len)
			.map(|ser| SerializeHelper::new(ser, param))
	}

	fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
	where
		I: IntoIterator,
		<I as IntoIterator>::Item: SerdeSerialize,
	{
		let param = self.param();
		let iter = iter
			.into_iter()
			.map(|item| SerializeSized::new(item, param));
		self.inner.collect_seq(iter)
	}

	fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
	where
		K: SerdeSerialize,
		V: SerdeSerialize,
		I: IntoIterator<Item = (K, V)>,
	{
		let param = self.param();
		let iter = iter
			.into_iter()
			.map(|(k, v)| (SerializeSized::new(k, param), SerializeSized::new(v, param)));
		self.inner.collect_map(iter)
	}

	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Display,
	{
		self.inner.collect_str(value)
	}

	fn is_human_readable(&self) -> bool {
		self.inner.is_human_readable()
	}
}

pub struct SerializeHelper<S> {
	inner: S,
	param: Param,
}

impl<S> SerializeHelper<S> {
	const fn new(inner: S, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<S: SerializeMap> SerializeMap for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner.serialize_key(&Serialize::new(key, self.param))
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_value(&Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}

	fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
	where
		K: ?Sized + SerdeSerialize,
		V: ?Sized + SerdeSerialize,
	{
		self.inner.serialize_entry(
			&Serialize::new(key, self.param),
			&Serialize::new(value, self.param),
		)
	}
}

impl<S: SerializeSeq> SerializeSeq for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_element(&Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}
}

impl<S: SerializeStruct> SerializeStruct for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_field(key, &Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}

	fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
		self.inner.skip_field(key)
	}
}

impl<S: SerializeStructVariant> SerializeStructVariant for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_field(key, &Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}

	fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
		self.inner.skip_field(key)
	}
}

impl<S: SerializeTuple> SerializeTuple for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_element(&Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}
}

impl<S: SerializeTupleStruct> SerializeTupleStruct for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_field(&Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}
}

impl<S: SerializeTupleVariant> SerializeTupleVariant for SerializeHelper<S> {
	type Error = S::Error;
	type Ok = S::Ok;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + SerdeSerialize,
	{
		self.inner
			.serialize_field(&Serialize::new(value, self.param))
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.inner.end()
	}
}

struct SerializeSized<T> {
	value: T,
	param: Param,
}

impl<T> SerializeSized<T> {
	const fn new(value: T, param: Param) -> Self {
		Self { value, param }
	}
}

impl<T: SerdeSerialize> SerdeSerialize for SerializeSized<T> {
	#[cfg(miri)]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SerdeSerializer,
	{
		SerdeSerialize::serialize(
			&self.value,
			Serializer {
				inner: serializer,
				red_zone: self.param.red_zone,
				stack_size: self.param.stack_size,
			},
		)
	}

	#[cfg(not(miri))]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SerdeSerializer,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			SerdeSerialize::serialize(
				&self.value,
				Serializer {
					inner: serializer,
					red_zone: self.param.red_zone,
					stack_size: self.param.stack_size,
				},
			)
		})
	}
}

struct Serialize<'a, T: ?Sized> {
	value: &'a T,
	param: Param,
}

impl<'a, T: ?Sized> Serialize<'a, T> {
	const fn new(value: &'a T, param: Param) -> Self {
		Self { value, param }
	}
}

impl<T> SerdeSerialize for Serialize<'_, T>
where
	T: ?Sized + SerdeSerialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: SerdeSerializer,
	{
		SerdeSerialize::serialize(
			self.value,
			Serializer {
				inner: serializer,
				red_zone: self.param.red_zone,
				stack_size: self.param.stack_size,
			},
		)
	}
}
