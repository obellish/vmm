use alloc::{borrow::ToOwned, collections::VecDeque, vec};

use serde::ser::{
	Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
	SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

use super::{Cow, Integer, Value};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ValueSerializer {
	use_indices: bool,
}

impl ValueSerializer {
	#[must_use]
	pub const fn new(use_indices: bool) -> Self {
		Self { use_indices }
	}
}

impl Serializer for ValueSerializer {
	type Error = Error;
	type Ok = Value<'static>;
	type SerializeMap = ValueMapSerializer;
	type SerializeSeq = ValueSeqSerializer;
	type SerializeStruct = ValueMapSerializer;
	type SerializeStructVariant = ValueMapVariantSerializer;
	type SerializeTuple = ValueSeqSerializer;
	type SerializeTupleStruct = ValueSeqSerializer;
	type SerializeTupleVariant = ValueSeqVariantSerializer;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Bool(v))
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(v.into())
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		let mut buffer = [0; 4];
		let s = v.encode_utf8(&mut buffer);
		self.serialize_str(s)
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		Ok(Value::String(Cow::Owned(v.to_owned())))
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Bytes(Cow::Owned(v.to_owned())))
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Null)
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		self.serialize_none()
	}

	fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
		self.serialize_unit()
	}

	fn serialize_unit_variant(
		self,
		_: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		if self.use_indices {
			Ok(variant_index.into())
		} else {
			Ok(Value::String(Cow::Borrowed(variant)))
		}
	}

	fn serialize_newtype_struct<T>(
		self,
		_: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	fn serialize_newtype_variant<T>(
		self,
		_: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = if self.use_indices {
			variant_index.into()
		} else {
			Value::String(Cow::Borrowed(variant))
		};

		let value = value.serialize(self)?;
		Ok(Value::Map(vec![(key, value)].into()))
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		let array = len.map_or_else(VecDeque::new, VecDeque::with_capacity);
		Ok(ValueSeqSerializer { inner: self, array })
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		self.serialize_seq(Some(len))
	}

	fn serialize_tuple_struct(
		self,
		_: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		self.serialize_tuple(len)
	}

	fn serialize_tuple_variant(
		self,
		_: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		let key = if self.use_indices {
			variant_index.into()
		} else {
			Value::String(Cow::Borrowed(variant))
		};

		Ok(ValueSeqVariantSerializer {
			inner: self,
			key,
			array: VecDeque::with_capacity(len),
		})
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		let map = len.map_or_else(VecDeque::new, VecDeque::with_capacity);
		Ok(ValueMapSerializer {
			inner: self,
			map,
			field_index: 0,
		})
	}

	fn serialize_struct(
		self,
		_: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		self.serialize_map(Some(len))
	}

	fn serialize_struct_variant(
		self,
		_: &'static str,
		variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		let key = if self.use_indices {
			variant_index.into()
		} else {
			Value::String(Cow::Borrowed(variant))
		};

		Ok(ValueMapVariantSerializer {
			inner: self,
			key,
			map: VecDeque::with_capacity(len),
			field_index: 0,
		})
	}

	fn is_human_readable(&self) -> bool {
		true
	}
}

#[derive(Debug)]
pub struct ValueSeqSerializer {
	inner: ValueSerializer,
	array: VecDeque<Value<'static>>,
}

impl SerializeSeq for ValueSeqSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.array.push_back(value.serialize(self.inner)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.array))
	}
}

impl SerializeTuple for ValueSeqSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.array.push_back(value.serialize(self.inner)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.array))
	}
}

impl SerializeTupleStruct for ValueSeqSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.array.push_back(value.serialize(self.inner)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.array))
	}
}

#[derive(Debug)]
pub struct ValueMapSerializer {
	inner: ValueSerializer,
	map: VecDeque<(Value<'static>, Value<'static>)>,
	field_index: u32,
}

impl SerializeMap for ValueMapSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = key.serialize(self.inner)?;
		self.map.push_back((key, Value::Null));
		Ok(())
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.map
			.back_mut()
			.expect("serialize_key is called before serialize_value")
			.1 = value.serialize(self.inner)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(self.map))
	}
}

impl SerializeStruct for ValueMapSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = if self.inner.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(self.field_index)))
		} else {
			Value::String(Cow::Borrowed(key))
		};

		self.field_index += 1;
		let value = value.serialize(self.inner)?;
		self.map.push_back((key, value));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(self.map))
	}

	fn skip_field(&mut self, _: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}

#[derive(Debug)]
pub struct ValueSeqVariantSerializer {
	inner: ValueSerializer,
	key: Value<'static>,
	array: VecDeque<Value<'static>>,
}

impl SerializeTupleVariant for ValueSeqVariantSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.array.push_back(value.serialize(self.inner)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(
			vec![(self.key, Value::Array(self.array))].into(),
		))
	}
}

#[derive(Debug)]
pub struct ValueMapVariantSerializer {
	inner: ValueSerializer,
	key: Value<'static>,
	map: VecDeque<(Value<'static>, Value<'static>)>,
	field_index: u32,
}

impl SerializeStructVariant for ValueMapVariantSerializer {
	type Error = Error;
	type Ok = Value<'static>;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = if self.inner.use_indices {
			Value::Integer(Integer::Unsigned(u128::from(self.field_index)))
		} else {
			Value::String(Cow::Borrowed(key))
		};

		self.field_index += 1;
		let value = value.serialize(self.inner)?;
		self.map.push_back((key, value));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Map(vec![(self.key, Value::Map(self.map))].into()))
	}

	fn skip_field(&mut self, _: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}
