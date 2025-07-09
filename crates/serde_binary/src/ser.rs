use serde::ser::{
	Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
	SerializeTupleStruct, SerializeTupleVariant, Serializer as SerdeSerializer,
};

use super::{Config, Error, Type, format::VarInt as _, io::Output};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serializer<O> {
	output: O,
	use_indices: bool,
}

impl<O> Serializer<O> {
	pub fn into_output(self) -> O {
		self.output
	}
}

impl<O: Output> Serializer<O> {
	pub fn new(output: O) -> Self {
		Self {
			output,
			use_indices: Config::default().use_indices,
		}
	}

	#[must_use]
	pub const fn use_indices(mut self, use_indices: bool) -> Self {
		self.use_indices = use_indices;
		self
	}
}

impl<'a, O: Output> SerdeSerializer for &'a mut Serializer<O> {
	type Error = Error;
	type Ok = ();
	type SerializeMap = Self;
	type SerializeSeq = Self;
	type SerializeStruct = StructSerializer<'a, O>;
	type SerializeStructVariant = StructSerializer<'a, O>;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		if v {
			self.output.write_byte(Type::True.into())
		} else {
			self.output.write_byte(Type::False.into())
		}
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::UnsignedInt.into())?;
		v.encode(&mut self.output)
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Float32.into())?;
		self.output.write_all(&v.to_le_bytes())
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Float64.into())?;
		self.output.write_all(&v.to_le_bytes())
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		let mut buffer = [0; 4];
		let s = v.encode_utf8(&mut buffer);
		self.serialize_str(s)
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::String.into())?;
		let bytes = v.as_bytes();
		bytes.len().encode(&mut self.output)?;
		self.output.write_all(bytes)
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Bytes.into())?;
		v.len().encode(&mut self.output)?;
		self.output.write_all(v)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::Null.into())
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
			variant_index.serialize(self)
		} else {
			variant.serialize(self)
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
		let use_indices = self.use_indices;
		let mut map = self.serialize_map(Some(1))?;
		if use_indices {
			map.serialize_entry(&variant_index, value)?;
		} else {
			map.serialize_entry(variant, value)?;
		}

		SerializeMap::end(self)
	}

	fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		self.output.write_byte(Type::SeqStart.into())?;
		Ok(self)
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
		_: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		if self.use_indices {
			variant_index.serialize(&mut *self)?;
		} else {
			variant.serialize(&mut *self)?;
		}

		self.output.write_byte(Type::SeqStart.into())?;
		Ok(self)
	}

	fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		Ok(self)
	}

	fn serialize_struct(
		self,
		_: &'static str,
		_: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		Ok(StructSerializer::new(self))
	}

	fn serialize_struct_variant(
		self,
		_: &'static str,
		variant_index: u32,
		variant: &'static str,
		_: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		self.output.write_byte(Type::MapStart.into())?;
		if self.use_indices {
			variant_index.serialize(&mut *self)?;
		} else {
			variant.serialize(&mut *self)?;
		}

		self.output.write_byte(Type::MapStart.into())?;
		Ok(StructSerializer::new(self))
	}

	#[cfg(feature = "alloc")]
	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + core::fmt::Display,
	{
		let s = alloc::string::ToString::to_string(value);
		self.serialize_str(&s)
	}

	#[cfg(not(feature = "alloc"))]
	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + core::fmt::Display,
	{
		use core::fmt::Write;

		#[repr(transparent)]
		struct CountWriter(usize);

		impl Write for CountWriter {
			fn write_str(&mut self, s: &str) -> core::fmt::Result {
				self.0 += s.len();
				Ok(())
			}
		}

		struct OutputWriter<'a, O>(&'a mut O);

		impl<O: Output> Write for OutputWriter<'_, O> {
			fn write_str(&mut self, s: &str) -> core::fmt::Result {
				self.0.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
			}
		}

		let mut counter = CountWriter(0);
		write!(&mut counter, "{value}")?;
		let len = counter.0;
		self.output.write_byte(Type::String.into())?;
		len.encode(&mut self.output)?;

		let mut writer = OutputWriter(&mut self.output);
		write!(&mut writer, "{value}")?;

		Ok(())
	}
}

impl<O: Output> SerializeMap for &mut Serializer<O> {
	type Error = Error;
	type Ok = ();

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		key.serialize(&mut **self)
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}
}

impl<O: Output> SerializeSeq for &mut Serializer<O> {
	type Error = Error;
	type Ok = ();

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())
	}
}

impl<O: Output> SerializeTuple for &mut Serializer<O> {
	type Error = Error;
	type Ok = ();

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())
	}
}

impl<O: Output> SerializeTupleStruct for &mut Serializer<O> {
	type Error = Error;
	type Ok = ();

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output.write_byte(Type::SeqEnd.into())
	}
}

impl<O: Output> SerializeTupleVariant for &mut Serializer<O> {
	type Error = Error;
	type Ok = ();

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(&mut **self)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.output
			.write_all(&[Type::SeqEnd.into(), Type::MapEnd.into()])
	}
}

#[derive(Debug)]
pub struct StructSerializer<'a, O> {
	serializer: &'a mut Serializer<O>,
	field_index: u32,
}

impl<'a, O> StructSerializer<'a, O> {
	const fn new(serializer: &'a mut Serializer<O>) -> Self {
		Self {
			serializer,
			field_index: 0,
		}
	}
}

impl<O: Output> SerializeStruct for StructSerializer<'_, O> {
	type Error = Error;
	type Ok = ();

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if self.serializer.use_indices {
			self.field_index.serialize(&mut *self.serializer)?;
		} else {
			key.serialize(&mut *self.serializer)?;
		}

		self.field_index += 1;
		value.serialize(&mut *self.serializer)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.serializer.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}

	fn skip_field(&mut self, _: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}

impl<O: Output> SerializeStructVariant for StructSerializer<'_, O> {
	type Error = Error;
	type Ok = ();

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if self.serializer.use_indices {
			self.field_index.serialize(&mut *self.serializer)?;
		} else {
			key.serialize(&mut *self.serializer)?;
		}

		self.field_index += 1;
		value.serialize(&mut *self.serializer)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.serializer
			.output
			.write_all(&[Type::MapEnd.into(); 2])?;
		Ok(())
	}

	fn skip_field(&mut self, _: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}
