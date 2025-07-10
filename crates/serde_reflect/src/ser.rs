use std::ops::{Deref, DerefMut};

use serde::ser::{
	Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
	SerializeTupleStruct, SerializeTupleVariant, Serializer as SerdeSerializer,
};

use super::{Error, Result, Samples, Tracer, Value, format::*};

pub struct Serializer<'a> {
	tracer: &'a mut Tracer,
	samples: &'a mut Samples,
}

impl<'a> Serializer<'a> {
	pub(crate) fn new(tracer: &'a mut Tracer, samples: &'a mut Samples) -> Self {
		Self { tracer, samples }
	}

	pub(crate) fn reborrow<'b: 'a>(&'b mut self) -> Serializer<'a> {
		Self::new(self.tracer, self.samples)
	}
}

impl<'a> SerdeSerializer for Serializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);
	type SerializeMap = MapSerializer<'a>;
	type SerializeSeq = SeqSerializer<'a>;
	type SerializeStruct = StructSerializer<'a>;
	type SerializeStructVariant = StructVariantSerializer<'a>;
	type SerializeTuple = TupleSerializer<'a>;
	type SerializeTupleStruct = TupleStructSerializer<'a>;
	type SerializeTupleVariant = TupleVariantSerializer<'a>;

	fn serialize_bool(self, v: bool) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_i8(self, v: i8) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_i16(self, v: i16) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_i32(self, v: i32) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_i64(self, v: i64) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_i128(self, v: i128) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_u8(self, v: u8) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_u16(self, v: u16) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_u32(self, v: u32) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_u64(self, v: u64) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_u128(self, v: u128) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_f32(self, v: f32) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_f64(self, v: f64) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_char(self, v: char) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v).into_format_and_value())
	}

	fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v.to_owned()).into_format_and_value())
	}

	fn serialize_bytes(self, v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(v.to_owned()).into_format_and_value())
	}

	fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
		Ok((Format::unknown(), Value::Option(None)))
	}

	fn serialize_some<T>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(self)?;

		Ok((
			Format::Option(Box::new(format)),
			Value::Option(Some(Box::new(value))),
		))
	}

	fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
		Ok(Value::from(()).into_format_and_value())
	}

	fn serialize_unit_struct(
		self,
		name: &'static str,
	) -> std::result::Result<Self::Ok, Self::Error> {
		self.tracer.record_container(
			self.samples,
			name,
			ContainerFormat::Unit,
			Value::Unit,
			false,
		)
	}

	fn serialize_unit_variant(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
	) -> std::result::Result<Self::Ok, Self::Error> {
		self.tracer.record_variant(
			self.samples,
			name,
			variant_index,
			variant,
			VariantFormat::Unit,
			Value::Unit,
		)
	}

	fn serialize_newtype_struct<T>(
		self,
		name: &'static str,
		value: &T,
	) -> std::result::Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(Serializer::new(self.tracer, self.samples))?;
		self.tracer.record_container(
			self.samples,
			name,
			ContainerFormat::Newtype(Box::new(format)),
			value,
			self.tracer.config.record_samples_for_newtype_structs,
		)
	}

	fn serialize_newtype_variant<T>(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> std::result::Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(Serializer::new(self.tracer, self.samples))?;
		self.tracer.record_variant(
			self.samples,
			name,
			variant_index,
			variant,
			VariantFormat::Newtype(Box::new(format)),
			value,
		)
	}

	fn serialize_seq(
		self,
		_: Option<usize>,
	) -> std::result::Result<Self::SerializeSeq, Self::Error> {
		Ok(SeqSerializer::new(self.tracer, self.samples))
	}
}

pub struct SeqSerializer<'a> {
	tracer: &'a mut Tracer,
	samples: &'a mut Samples,
	format: Format,
	values: Vec<Value>,
}

impl<'a> SeqSerializer<'a> {
	fn new(tracer: &'a mut Tracer, samples: &'a mut Samples) -> Self {
		Self {
			tracer,
			samples,
			format: Format::unknown(),
			values: Vec::new(),
		}
	}
}

impl<'a> SerializeSeq for SeqSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(Serializer::new(self.tracer, self.samples))?;
		self.format.unify(format)?;
		self.values.push(value);
		Ok(())
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		Ok((Format::Seq(Box::new(self.format)), Value::Seq(self.values)))
	}
}

pub struct TupleSerializer<'a> {
	tracer: &'a mut Tracer,
	samples: &'a mut Samples,
	formats: Vec<Format>,
	values: Vec<Value>,
}

impl<'a> SerializeTuple for TupleSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(Serializer::new(self.tracer, self.samples))?;
		self.formats.push(format);
		self.values.push(value);
		Ok(())
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		Ok((Format::Tuple(self.formats), Value::Seq(self.values)))
	}
}

pub struct TupleStructSerializer<'a> {
	name: &'static str,
	inner: TupleSerializer<'a>,
}

impl<'a> SerializeTupleStruct for TupleStructSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.inner.serialize_element(value)
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		let format = ContainerFormat::Tuple(self.inner.formats);
		let value = Value::Seq(self.inner.values);
		self.inner.tracer.record_container(
			self.inner.samples,
			self.name,
			format,
			value,
			self.inner.tracer.config.record_samples_for_tuple_structs,
		)
	}
}

pub struct TupleVariantSerializer<'a> {
	variant_index: u32,
	variant_name: &'static str,
	inner: TupleStructSerializer<'a>,
}

impl<'a> SerializeTupleVariant for TupleVariantSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.inner.serialize_field(value)
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		let variant = VariantFormat::Tuple(self.inner.inner.formats);
		let value = Value::Seq(self.inner.inner.values);
		self.inner.inner.tracer.record_variant(
			self.inner.inner.samples,
			self.inner.name,
			self.variant_index,
			self.variant_name,
			variant,
			value,
		)
	}
}

pub struct MapSerializer<'a> {
	tracer: &'a mut Tracer,
	samples: &'a mut Samples,
	key_format: Format,
	value_format: Format,
	values: Vec<Value>,
}

impl<'a> SerializeMap for MapSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = key.serialize(Serializer::new(self.tracer, self.samples))?;
		self.key_format.unify(format)?;
		self.values.push(value);
		Ok(())
	}

	fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(Serializer::new(self.tracer, self.samples))?;
		self.value_format.unify(format)?;
		self.values.push(value);
		Ok(())
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		let format = Format::Map {
			key: Box::new(self.key_format),
			value: Box::new(self.value_format),
		};

		let value = Value::Seq(self.values);
		Ok((format, value))
	}
}

pub struct StructSerializer<'a> {
	tracer: &'a mut Tracer,
	samples: &'a mut Samples,
	name: &'static str,
	fields: Vec<Named<Format>>,
	values: Vec<Value>,
}

impl<'a> SerializeStruct for StructSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_field<T>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let (format, value) = value.serialize(Serializer::new(self.tracer, self.samples))?;
		self.fields.push(Named::new(key.to_owned(), format));
		self.values.push(value);

		Ok(())
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		let format = ContainerFormat::Struct(self.fields);
		let value = Value::Seq(self.values);
		self.tracer.record_container(
			self.samples,
			self.name,
			format,
			value,
			self.tracer.config.record_samples_for_structs,
		)
	}
}

pub struct StructVariantSerializer<'a> {
	inner: StructSerializer<'a>,
	variant_index: u32,
	variant_name: &'static str,
}

impl<'a> SerializeStructVariant for StructVariantSerializer<'a> {
	type Error = Error;
	type Ok = (Format, Value);

	fn serialize_field<T>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> std::result::Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.inner.serialize_field(key, value)
	}

	fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
		let variant = VariantFormat::Struct(self.inner.fields);
		let value = Value::Seq(self.inner.values);

		self.inner.tracer.record_variant(
			self.inner.samples,
			self.inner.name,
			self.variant_index,
			self.variant_name,
			variant,
			value,
		)
	}
}
