mod error;

use alloc::{borrow::ToOwned, string::ToString};
use core::fmt::Debug;

use serde::{
	Serialize,
	ser::{
		SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
		SerializeTupleStruct, SerializeTupleVariant, Serializer as SerdeSerializer,
	},
};
use vmm_binary_io::Write;
use vmm_binary_ll::{Encoder, Header, simple, tag};

pub use self::error::*;

#[repr(transparent)]
struct Serializer<W>(Encoder<W>);

impl<W> From<Encoder<W>> for Serializer<W> {
	fn from(value: Encoder<W>) -> Self {
		Self(value)
	}
}

impl<W> From<W> for Serializer<W> {
	fn from(value: W) -> Self {
		Self(value.into())
	}
}

impl<'a, W> SerdeSerializer for &'a mut Serializer<W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();
	type SerializeMap = CollectionSerializer<'a, W>;
	type SerializeSeq = CollectionSerializer<'a, W>;
	type SerializeStruct = CollectionSerializer<'a, W>;
	type SerializeStructVariant = CollectionSerializer<'a, W>;
	type SerializeTuple = CollectionSerializer<'a, W>;
	type SerializeTupleStruct = CollectionSerializer<'a, W>;
	type SerializeTupleVariant = CollectionSerializer<'a, W>;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(self.0.push(if v {
			Header::Simple(simple::TRUE)
		} else {
			Header::Simple(simple::FALSE)
		})?)
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		self.serialize_i64(v.into())
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		self.serialize_i64(v.into())
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		self.serialize_i64(v.into())
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		Ok(self.0.push(if v.is_negative() {
			Header::Negative(v as u64 ^ !0)
		} else {
			Header::Positive(v as u64)
		})?)
	}

	fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
		let (tag, raw) = if v.is_negative() {
			(tag::BIGNEG, v as u128 ^ !0)
		} else {
			(tag::BIGPOS, v as u128)
		};

		match (tag, u64::try_from(raw)) {
			(tag::BIGPOS, Ok(x)) => return Ok(self.0.push(Header::Positive(x))?),
			(tag::BIGNEG, Ok(x)) => return Ok(self.0.push(Header::Negative(x))?),
			_ => {}
		}

		let bytes = raw.to_be_bytes();

		let mut slice = &bytes[..];
		while !slice.is_empty() && matches!(slice[0], 0) {
			slice = &slice[1..];
		}

		self.0.push(Header::Tag(tag))?;
		self.0.push(Header::Bytes(Some(slice.len())))?;
		Ok(self.0.write_all(slice)?)
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		self.serialize_u64(v.into())
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		self.serialize_u64(v.into())
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		self.serialize_u64(v.into())
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		Ok(self.0.push(Header::Positive(v))?)
	}

	fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
		if let Ok(v) = u64::try_from(v) {
			return self.serialize_u64(v);
		}

		let bytes = v.to_be_bytes();

		let mut slice = &bytes[..];
		while !slice.is_empty() && matches!(slice[0], 0) {
			slice = &slice[1..];
		}

		self.0.push(Header::Tag(tag::BIGPOS))?;
		self.0.push(Header::Bytes(Some(slice.len())))?;
		Ok(self.0.write_all(slice)?)
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		self.serialize_f64(v.into())
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(self.0.push(Header::Float(v))?)
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		self.serialize_str(&v.to_string())
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		let bytes = v.as_bytes();
		self.0.push(Header::Text(bytes.len().into()))?;
		Ok(self.0.write_all(bytes)?)
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		self.0.push(Header::Bytes(v.len().into()))?;
		Ok(self.0.write_all(v)?)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.0.push(Header::Simple(simple::NULL))?)
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
		_: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		self.serialize_str(variant)
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
		name: &'static str,
		_: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if !matches!(name, "@@TAG@@") || !matches!(variant, "@@UNTAGGED@@") {
			self.0.push(Header::Map(Some(1)))?;
			self.serialize_str(variant)?;
		}

		value.serialize(self)
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		self.0.push(Header::Array(len))?;
		Ok(CollectionSerializer {
			encoder: self,
			ending: len.is_none(),
			tag: false,
		})
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
		name: &'static str,
		_: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		if (name, variant) == ("@@TAG@@", "@@TAGGED@@") {
			Ok(CollectionSerializer {
				encoder: self,
				ending: false,
				tag: true,
			})
		} else {
			self.0.push(Header::Map(Some(1)))?;
			self.serialize_str(variant)?;
			self.0.push(Header::Array(Some(len)))?;
			Ok(CollectionSerializer {
				encoder: self,
				ending: false,
				tag: false,
			})
		}
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		self.0.push(Header::Map(len))?;
		Ok(CollectionSerializer {
			encoder: self,
			ending: len.is_none(),
			tag: false,
		})
	}

	fn serialize_struct(
		self,
		_: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		self.0.push(Header::Map(Some(len)))?;
		Ok(CollectionSerializer {
			encoder: self,
			ending: false,
			tag: false,
		})
	}

	fn serialize_struct_variant(
		self,
		_: &'static str,
		_: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		self.0.push(Header::Map(Some(1)))?;
		self.serialize_str(variant)?;
		self.0.push(Header::Map(Some(len)))?;
		Ok(CollectionSerializer {
			encoder: self,
			ending: false,
			tag: false,
		})
	}

	fn is_human_readable(&self) -> bool {
		false
	}
}

struct CollectionSerializer<'a, W> {
	encoder: &'a mut Serializer<W>,
	ending: bool,
	tag: bool,
}

impl<W> SerializeMap for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		key.serialize(&mut *self.encoder)
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut *self.encoder)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

impl<W> SerializeSeq for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut *self.encoder)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

impl<W> SerializeStruct for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		key.serialize(&mut *self.encoder)?;
		value.serialize(&mut *self.encoder)?;

		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

impl<W> SerializeStructVariant for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		key.serialize(&mut *self.encoder)?;
		value.serialize(&mut *self.encoder)?;

		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

impl<W> SerializeTuple for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut *self.encoder)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

impl<W> SerializeTupleStruct for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut *self.encoder)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

impl<W> SerializeTupleVariant for CollectionSerializer<'_, W>
where
	W: Write,
	W::Error: Debug,
{
	type Error = Error<W::Error>;
	type Ok = ();

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if !self.tag {
			return value.serialize(&mut *self.encoder);
		}

		self.tag = false;
		match value.serialize(super::tag::Serializer) {
			Ok(x) => Ok(self.encoder.0.push(Header::Tag(x))?),
			_ => Err(Error::Value("expected tag".to_owned())),
		}
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		end(self)
	}
}

pub fn into_writer<T, W>(value: &T, writer: W) -> Result<(), Error<W::Error>>
where
	T: ?Sized + Serialize,
	W: Write,
	W::Error: Debug,
{
	let mut encoder = Serializer::from(writer);
	value.serialize(&mut encoder)
}

#[cfg(feature = "std")]
pub fn into_vec<T>(
	value: &T,
) -> Result<alloc::vec::Vec<u8>, Error<<alloc::vec::Vec<u8> as Write>::Error>>
where
	T: ?Sized + Serialize,
{
	let mut vector = alloc::vec::Vec::new();
	into_writer(value, &mut vector)?;
	Ok(vector)
}

fn end<W>(collection_serializer: CollectionSerializer<'_, W>) -> Result<(), Error<W::Error>>
where
	W: Write,
	W::Error: Debug,
{
	if collection_serializer.ending {
		collection_serializer.encoder.0.push(Header::Break)?;
	}

	Ok(())
}
