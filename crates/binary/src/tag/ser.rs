use core::{
	error::Error as CoreError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use serde::{
	Serializer as SerdeSerializer,
	ser::{
		Error as SerError, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
		SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
	},
};

#[derive(Debug)]
pub(crate) struct Error;

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("an error occurred")
	}
}

impl CoreError for Error {}

impl SerError for Error {
	fn custom<T>(_: T) -> Self
	where
		T: Display,
	{
		Self
	}
}

pub(crate) struct Serializer;

impl SerdeSerializer for Serializer {
	type Error = Error;
	type Ok = u64;
	type SerializeMap = Self;
	type SerializeSeq = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;

	fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_i128(self, _: i128) -> Result<Self::Ok, Self::Error> {
		Err(Error)
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
		Ok(v)
	}

	fn serialize_u128(self, _: u128) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_unit_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}

	fn serialize_newtype_struct<T>(self, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn serialize_newtype_variant<T>(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Err(Error)
	}

	fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
		Err(Error)
	}

	fn serialize_tuple_struct(
		self,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		Err(Error)
	}

	fn serialize_tuple_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		Err(Error)
	}

	fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Err(Error)
	}

	fn serialize_struct(
		self,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		Err(Error)
	}

	fn serialize_struct_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		Err(Error)
	}

	fn is_human_readable(&self) -> bool {
		false
	}
}

impl SerializeSeq for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_element<T>(&mut self, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}

impl SerializeTuple for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_element<T>(&mut self, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}

impl SerializeTupleStruct for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_field<T>(&mut self, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}

impl SerializeTupleVariant for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_field<T>(&mut self, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}

impl SerializeMap for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_key<T>(&mut self, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn serialize_value<T>(&mut self, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}

impl SerializeStruct for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}

impl SerializeStructVariant for Serializer {
	type Error = Error;
	type Ok = u64;

	fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(Error)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Err(Error)
	}
}
