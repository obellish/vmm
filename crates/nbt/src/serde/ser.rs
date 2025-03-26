use std::{hash::Hash, marker::PhantomData};

use serde::{
	Serialize, Serializer,
	ser::{Impossible, SerializeMap, SerializeSeq, SerializeStruct},
};

use crate::{
	Compound, Error, List, Value,
	conv::{i8_slice_as_u8_slice, u8_vec_into_i8_vec},
};

impl<Str> Serialize for Value<Str>
where
	Str: Hash + Ord + Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			Self::Byte(v) => serializer.serialize_i8(*v),
			Self::Short(v) => serializer.serialize_i16(*v),
			Self::Int(v) => serializer.serialize_i32(*v),
			Self::Long(v) => serializer.serialize_i64(*v),
			Self::Float(v) => serializer.serialize_f32(*v),
			Self::Double(v) => serializer.serialize_f64(*v),
			Self::ByteArray(v) => serializer.serialize_bytes(i8_slice_as_u8_slice(v)),
			Self::String(v) => v.serialize(serializer),
			Self::List(v) => v.serialize(serializer),
			Self::Compound(v) => v.serialize(serializer),
			Self::IntArray(v) => v.serialize(serializer),
			Self::LongArray(v) => v.serialize(serializer),
		}
	}
}

impl<Str> Serialize for List<Str>
where
	Str: Hash + Ord + Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			Self::End => serializer.serialize_seq(Some(0))?.end(),
			Self::Byte(v) => v.serialize(serializer),
			Self::Short(v) => v.serialize(serializer),
			Self::Int(v) => v.serialize(serializer),
			Self::Long(v) => v.serialize(serializer),
			Self::Float(v) => v.serialize(serializer),
			Self::Double(v) => v.serialize(serializer),
			Self::ByteArray(v) => v.serialize(serializer),
			Self::String(v) => v.serialize(serializer),
			Self::List(v) => v.serialize(serializer),
			Self::Compound(v) => v.serialize(serializer),
			Self::IntArray(v) => v.serialize(serializer),
			Self::LongArray(v) => v.serialize(serializer),
		}
	}
}

macro_rules! unsupported {
	($lit:literal) => {
		::std::result::Result::Err($crate::Error::r#static(concat!("unsupported type: ", $lit)))
	};
}

#[derive(Debug)]
#[repr(transparent)]
pub struct CompoundSerializer;

impl Serializer for CompoundSerializer {
	type Error = Error;
	type Ok = Compound;
	type SerializeMap = GenericSerializeMap<Self::Ok>;
	type SerializeSeq = Impossible<Self::Ok, Self::Error>;
	type SerializeStruct = GenericSerializeStruct<Self::Ok>;
	type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
	type SerializeTuple = Impossible<Self::Ok, Self::Error>;
	type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
	type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

	fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
		unsupported!("bool")
	}

	fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
		unsupported!("i8")
	}

	fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
		unsupported!("i16")
	}

	fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
		unsupported!("i32")
	}

	fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
		unsupported!("i64")
	}

	fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
		unsupported!("u8")
	}

	fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
		unsupported!("u16")
	}

	fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
		unsupported!("u32")
	}

	fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
		unsupported!("u64")
	}

	fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
		unsupported!("f32")
	}

	fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
		unsupported!("f64")
	}

	fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
		unsupported!("char")
	}

	fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
		unsupported!("str")
	}

	fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
		unsupported!("bytes")
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		unsupported!("none")
	}

	fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		unsupported!("some")
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		unsupported!("unit")
	}

	fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
		unsupported!("unit struct")
	}

	fn serialize_unit_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		unsupported!("unit variant")
	}

	fn serialize_newtype_struct<T>(self, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		unsupported!("newtype struct")
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
		unsupported!("newtype variant")
	}

	fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		unsupported!("seq")
	}

	fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
		unsupported!("tuple")
	}

	fn serialize_tuple_struct(
		self,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		unsupported!("tuple struct")
	}

	fn serialize_tuple_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		unsupported!("tuple variant")
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Ok(GenericSerializeMap::new(len))
	}

	fn serialize_struct(
		self,
		_: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		Ok(GenericSerializeStruct::new(len))
	}

	fn serialize_struct_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		unsupported!("struct variant")
	}
}

#[doc(hidden)]
#[derive(Debug)]
pub struct GenericSerializeMap<Ok> {
	key: Option<String>,
	res: Compound,
	marker: PhantomData<Ok>,
}

impl<Ok> GenericSerializeMap<Ok> {
	#[must_use]
	pub fn new(len: Option<usize>) -> Self {
		Self {
			key: None,
			res: Compound::with_capacity(len.unwrap_or_default()),
			marker: PhantomData,
		}
	}
}

impl<Ok> SerializeMap for GenericSerializeMap<Ok>
where
	Compound: Into<Ok>,
{
	type Error = Error;
	type Ok = Ok;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		debug_assert!(
			self.key.is_none(),
			"call to `serialize_key` must be followed by `serialize_value`"
		);

		match key.serialize(ValueSerializer)? {
			Value::String(s) => {
				self.key = Some(s);
				Ok(())
			}
			_ => Err(Error::r#static("invalid map key type (expected string)")),
		}
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.res.into())
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = self
			.key
			.take()
			.expect("missing previous call to `serialize_key`");
		self.res.insert(key, value.serialize(ValueSerializer)?);
		Ok(())
	}
}

#[derive(Debug)]
#[doc(hidden)]
#[repr(transparent)]
pub struct GenericSerializeStruct<Ok> {
	c: Compound,
	marker: PhantomData<Ok>,
}

impl<Ok> GenericSerializeStruct<Ok> {
	fn new(len: usize) -> Self {
		Self {
			c: Compound::with_capacity(len),
			marker: PhantomData,
		}
	}
}

impl<Ok> SerializeStruct for GenericSerializeStruct<Ok>
where
	Compound: Into<Ok>,
{
	type Error = Error;
	type Ok = Ok;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.c.insert(key, value.serialize(ValueSerializer)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.c.into())
	}
}

struct ValueSerializer;

impl Serializer for ValueSerializer {
	type Error = Error;
	type Ok = Value;
	type SerializeMap = GenericSerializeMap<Self::Ok>;
	type SerializeSeq = ValueSerializeSeq;
	type SerializeStruct = GenericSerializeStruct<Self::Ok>;
	type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
	type SerializeTuple = Impossible<Self::Ok, Self::Error>;
	type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
	type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Byte(v.into()))
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Byte(v))
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Short(v))
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Int(v))
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Long(v))
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Byte(v as i8))
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Short(v as i16))
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Int(v as i32))
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Long(v as i64))
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Float(v))
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Double(v))
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		Ok(Value::String(v.into()))
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		Ok(Value::String(v.to_owned()))
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		Ok(Value::ByteArray(u8_vec_into_i8_vec(v.to_owned())))
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		unsupported!("none")
	}

	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		unsupported!("unit")
	}

	fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
		unsupported!("unit struct")
	}

	fn serialize_unit_variant(
		self,
		_: &'static str,
		_: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		Ok(Value::String(variant.to_owned()))
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
		_: u32,
		_: &'static str,
		_: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		unsupported!("newtype variant")
	}

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Ok(ValueSerializeSeq::End {
			len: len.unwrap_or_default(),
		})
	}

	fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
		unsupported!("tuple")
	}

	fn serialize_tuple_struct(
		self,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		unsupported!("tuple struct")
	}

	fn serialize_tuple_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		unsupported!("tuple variant")
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Ok(GenericSerializeMap::new(len))
	}

	fn serialize_struct(
		self,
		_: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		Ok(GenericSerializeStruct::new(len))
	}

	fn serialize_struct_variant(
		self,
		_: &'static str,
		_: u32,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		unsupported!("struct variant")
	}
}

enum ValueSerializeSeq {
	End { len: usize },
	Byte(Vec<i8>),
	Short(Vec<i16>),
	Int(Vec<i32>),
	Long(Vec<i64>),
	Float(Vec<f32>),
	Double(Vec<f64>),
	ByteArray(Vec<Vec<i8>>),
	String(Vec<String>),
	List(Vec<List>),
	Compound(Vec<Compound>),
	IntArray(Vec<Vec<i32>>),
	LongArray(Vec<Vec<i64>>),
}

impl SerializeSeq for ValueSerializeSeq {
	type Error = Error;
	type Ok = Value;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		macro_rules! serialize_variant {
			($variant:ident, $vec:ident, $elem:ident) => {{
				match $elem.serialize($crate::serde::ser::ValueSerializer)? {
					$crate::Value::$variant(val) => {
						$vec.push(val);
						::std::result::Result::Ok(())
					}
					_ => ::std::result::Result::Err($crate::Error::r#static(concat!(
						"heterogeneous NBT list (expected `",
						stringify!($variant),
						"` element)"
					))),
				}
			}};
		}

		match self {
			Self::End { len } => {
				fn vec<T>(elem: T, len: usize) -> Vec<T> {
					let mut vec = Vec::with_capacity(len);
					vec.push(elem);
					vec
				}

				*self = match value.serialize(ValueSerializer)? {
					Value::Byte(v) => Self::Byte(vec(v, *len)),
					Value::Short(v) => Self::Short(vec(v, *len)),
					Value::Int(v) => Self::Int(vec(v, *len)),
					Value::Long(v) => Self::Long(vec(v, *len)),
					Value::Float(v) => Self::Float(vec(v, *len)),
					Value::Double(v) => Self::Double(vec(v, *len)),
					Value::ByteArray(v) => Self::ByteArray(vec(v, *len)),
					Value::String(v) => Self::String(vec(v, *len)),
					Value::List(v) => Self::List(vec(v, *len)),
					Value::Compound(v) => Self::Compound(vec(v, *len)),
					Value::IntArray(v) => Self::IntArray(vec(v, *len)),
					Value::LongArray(v) => Self::LongArray(vec(v, *len)),
				};
				Ok(())
			}
			Self::Byte(v) => serialize_variant!(Byte, v, value),
			Self::Short(v) => serialize_variant!(Short, v, value),
			Self::Int(v) => serialize_variant!(Int, v, value),
			Self::Long(v) => serialize_variant!(Long, v, value),
			Self::Float(v) => serialize_variant!(Float, v, value),
			Self::Double(v) => serialize_variant!(Double, v, value),
			Self::ByteArray(v) => serialize_variant!(ByteArray, v, value),
			Self::String(v) => serialize_variant!(String, v, value),
			Self::List(v) => serialize_variant!(List, v, value),
			Self::Compound(v) => serialize_variant!(Compound, v, value),
			Self::IntArray(v) => serialize_variant!(IntArray, v, value),
			Self::LongArray(v) => serialize_variant!(LongArray, v, value),
		}
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(match self {
			Self::End { .. } => List::End.into(),
			Self::Byte(v) => v.into(),
			Self::Short(v) => List::Short(v).into(),
			Self::Int(v) => v.into(),
			Self::Long(v) => List::Long(v).into(),
			Self::Float(v) => List::Float(v).into(),
			Self::Double(v) => List::Double(v).into(),
			Self::ByteArray(v) => List::ByteArray(v).into(),
			Self::String(v) => List::String(v).into(),
			Self::List(v) => List::List(v).into(),
			Self::Compound(v) => List::Compound(v).into(),
			Self::IntArray(v) => List::IntArray(v).into(),
			Self::LongArray(v) => List::LongArray(v).into(),
		})
	}
}
