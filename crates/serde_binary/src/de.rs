use core::{mem::size_of, str};

use serde::{
	de::{
		DeserializeSeed, Deserializer as SerdeDeserializer, EnumAccess, Error as DeError,
		IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
	},
	forward_to_deserialize_any,
};

use super::{Buffer, Error, Result, Type, VarInt, io::Input};

#[derive(Debug)]
pub struct Deserializer<I, B = ()> {
	input: I,
	buffer: Option<B>,
}

impl<I, B> Deserializer<I, B> {
	pub fn into_input(self) -> I {
		self.input
	}

	pub fn into_parts(self) -> (I, Option<B>) {
		(self.input, self.buffer)
	}
}

impl<'de, I, B: Buffer> Deserializer<I, B>
where
	I: Input<'de>,
{
	fn peek_type(&mut self) -> Result<Type> {
		let byte = self.input.peek_byte()?;
		Type::try_from(byte)
	}

	fn deserialize_null<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		_ = self.input.read_byte()?;
		visitor.visit_none()
	}

	fn reset_buffer(&mut self) {
		if let Some(buffer) = self.buffer.as_mut() {
			buffer.clear();
		}
	}

	fn buffer_slice(&self) -> Result<&[u8]> {
		Ok(self
			.buffer
			.as_ref()
			.ok_or(Error::BufferTooSmall)?
			.as_slice())
	}

	fn read_bytes<'s>(&'s mut self, len: usize) -> Result<&'s [u8]>
	where
		'de: 's,
	{
		self.reset_buffer();
		if let Some(data) = self.input.read_bytes(len, self.buffer.as_mut())? {
			Ok(data)
		} else {
			self.buffer_slice()
		}
	}

	fn deserialize_ptr<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let number = usize::decode(&mut self.input)?;
				match size_of::<usize>() {
					1 => visitor.visit_u8(number as u8),
					2 => visitor.visit_u16(number as u16),
					4 => visitor.visit_u32(number as u32),
					8 => visitor.visit_u64(number as u64),
					16 => visitor.visit_u128(number as u128),
					_ => unreachable!("unknown usize size"),
				}
			}
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let number = isize::decode(&mut self.input)?;
				match size_of::<isize>() {
					1 => visitor.visit_i8(number as i8),
					2 => visitor.visit_i16(number as i16),
					4 => visitor.visit_i32(number as i32),
					8 => visitor.visit_i64(number as i64),
					16 => visitor.visit_i128(number as i128),
					_ => unreachable!("unknown isize size"),
				}
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt, Type::SignedInt])),
		}
	}

	fn deserialize_float<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::Float32 => {
				_ = self.input.read_byte()?;
				let mut bytes = [0; 4];
				self.input.read_exact(&mut bytes)?;
				let value = f32::from_le_bytes(bytes);
				visitor.visit_f32(value)
			}
			Type::Float64 => {
				_ = self.input.read_byte()?;
				let mut bytes = [0; 8];
				self.input.read_exact(&mut bytes)?;
				let value = f64::from_le_bytes(bytes);
				visitor.visit_f64(value)
			}
			_ => Err(Error::WrongType(
				t,
				&[Type::Float16, Type::Float32, Type::Float64, Type::Float128],
			)),
		}
	}

	fn deserialize_unsigned_int<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u128::decode(&mut self.input)?;
				if value <= u128::from(u8::MAX) {
					visitor.visit_u8(value as u8)
				} else if value <= u128::from(u16::MAX) {
					visitor.visit_u16(value as u16)
				} else if value <= u128::from(u32::MAX) {
					visitor.visit_u32(value as u32)
				} else if value <= u128::from(u64::MAX) {
					visitor.visit_u64(value as u64)
				} else {
					visitor.visit_u128(value)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	fn deserialize_signed_int<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			#[allow(clippy::cast_lossless, reason = "we won't change it")]
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i128::decode(&mut self.input)?;
				if (i8::MIN as i128..=i8::MAX as i128).contains(&value) {
					visitor.visit_i8(value as i8)
				} else if (i16::MIN as i128..=i16::MAX as i128).contains(&value) {
					visitor.visit_i16(value as i16)
				} else if (i32::MIN as i128..=i32::MAX as i128).contains(&value) {
					visitor.visit_i32(value as i32)
				} else if (i64::MIN as i128..=i64::MAX as i128).contains(&value) {
					visitor.visit_i64(value as i64)
				} else {
					visitor.visit_i128(value)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}
}

impl<'de, I> Deserializer<I>
where
	I: Input<'de>,
{
	pub const fn new(input: I) -> Self {
		Self {
			input,
			buffer: None,
		}
	}

	pub fn and_with_buffer<B: Buffer>(self, buffer: B) -> Deserializer<I, B> {
		Deserializer {
			input: self.input,
			buffer: Some(buffer),
		}
	}
}

impl<'de, I, B: Buffer> SerdeDeserializer<'de> for &mut Deserializer<I, B>
where
	I: Input<'de>,
{
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_unit(visitor),
			Type::True | Type::False => self.deserialize_bool(visitor),
			Type::UnsignedInt => self.deserialize_unsigned_int(visitor),
			Type::SignedInt => self.deserialize_signed_int(visitor),
			Type::Float16 | Type::Float32 | Type::Float64 | Type::Float128 => {
				self.deserialize_float(visitor)
			}
			Type::Bytes => self.deserialize_byte_buf(visitor),
			Type::String => self.deserialize_string(visitor),
			Type::SeqStart => self.deserialize_seq(visitor),
			Type::MapStart => self.deserialize_map(visitor),
			Type::SeqEnd | Type::MapEnd => Err(Error::WrongType(
				t,
				&[
					Type::Null,
					Type::False,
					Type::True,
					Type::UnsignedInt,
					Type::SignedInt,
					Type::Float16,
					Type::Float32,
					Type::Float64,
					Type::Float128,
					Type::Bytes,
					Type::String,
					Type::SeqStart,
					Type::MapStart,
				],
			)),
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => {
				_ = self.input.read_byte()?;
				visitor.visit_unit()
			}
			_ => Err(Error::WrongType(t, &[Type::Null])),
		}
	}

	fn deserialize_unit_struct<V>(
		self,
		_: &'static str,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_bool<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::False | Type::True => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(matches!(t, Type::True))
			}
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt | Type::SignedInt => self.deserialize_ptr(visitor),
			_ => Err(Error::WrongType(t, &[Type::False, Type::True])),
		}
	}

	fn deserialize_i8<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i8::decode(&mut self.input)?;
				visitor.visit_i8(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	fn deserialize_i16<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i16::decode(&mut self.input)?;
				visitor.visit_i16(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	fn deserialize_i32<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i32::decode(&mut self.input)?;
				visitor.visit_i32(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	fn deserialize_i64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i64::decode(&mut self.input)?;
				visitor.visit_i64(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	fn deserialize_i128<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::SignedInt => {
				_ = self.input.read_byte()?;
				let value = i128::decode(&mut self.input)?;
				visitor.visit_i128(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SignedInt])),
		}
	}

	fn deserialize_u8<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u8::decode(&mut self.input)?;
				visitor.visit_u8(value)
			}
			Type::True | Type::False => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(matches!(t, Type::True))
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	fn deserialize_u16<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u16::decode(&mut self.input)?;
				visitor.visit_u16(value)
			}
			Type::True | Type::False => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(matches!(t, Type::True))
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	fn deserialize_u32<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u32::decode(&mut self.input)?;
				visitor.visit_u32(value)
			}
			Type::True | Type::False => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(matches!(t, Type::True))
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	fn deserialize_u64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u64::decode(&mut self.input)?;
				visitor.visit_u64(value)
			}
			Type::True | Type::False => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(matches!(t, Type::True))
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	fn deserialize_u128<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let value = u128::decode(&mut self.input)?;
				visitor.visit_u128(value)
			}
			Type::True | Type::False => {
				_ = self.input.read_byte()?;
				visitor.visit_bool(matches!(t, Type::True))
			}
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt])),
		}
	}

	fn deserialize_f32<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_float(visitor)
	}

	fn deserialize_f64<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_float(visitor)
	}

	fn deserialize_char<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let s = str::from_utf8(bytes)?;

				let mut chars = s.chars();
				let c = chars.next().ok_or(Error::NotOneChar)?;
				if chars.next().is_some() {
					return Err(Error::NotOneChar);
				}

				visitor.visit_char(c)
			}
			_ => Err(Error::WrongType(t, &[Type::String])),
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;

				self.reset_buffer();
				let borrowed = self.input.read_bytes(len, self.buffer.as_mut())?;
				if let Some(borrowed) = borrowed {
					let s = str::from_utf8(borrowed)?;
					visitor.visit_borrowed_str(s)
				} else {
					let s = str::from_utf8(self.buffer_slice()?)?;
					visitor.visit_str(s)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::String])),
		}
	}

	fn deserialize_string<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_identifier<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::UnsignedInt => self.deserialize_u32(visitor),
			Type::String => self.deserialize_str(visitor),
			_ => Err(Error::WrongType(t, &[Type::UnsignedInt, Type::String])),
		}
	}

	fn deserialize_bytes<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::Bytes | Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;

				self.reset_buffer();
				let borrowed = self.input.read_bytes(len, self.buffer.as_mut())?;
				if let Some(borrowed) = borrowed {
					visitor.visit_borrowed_bytes(borrowed)
				} else {
					visitor.visit_bytes(self.buffer_slice()?)
				}
			}
			_ => Err(Error::WrongType(t, &[Type::Bytes])),
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	fn deserialize_option<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			_ => visitor.visit_some(self),
		}
	}

	fn deserialize_newtype_struct<V>(
		self,
		_: &'static str,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::SeqStart => {
				_ = self.input.read_byte()?;
				let value = visitor.visit_seq(SequenceDeserializer(self))?;

				let byte = self.input.read_byte()?;
				let t = Type::try_from(byte)?;
				if matches!(t, Type::SeqEnd) {
					Ok(value)
				} else {
					Err(Error::WrongType(t, &[Type::SeqEnd]))
				}
			}
			Type::Bytes => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let value = visitor.visit_seq(ByteSequenceDeserializer(bytes))?;
				Ok(value)
			}
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let s = str::from_utf8(bytes)?;
				let value = visitor.visit_seq(CharSequenceDeserializer(s.chars()))?;
				Ok(value)
			}
			_ => Err(Error::WrongType(t, &[Type::SeqStart])),
		}
	}

	fn deserialize_tuple<V>(
		self,
		_: usize,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_: &'static str,
		len: usize,
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_tuple(len, visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::MapStart => {
				_ = self.input.read_byte()?;
				let value = visitor.visit_map(MapDeserializer(self))?;

				let byte = self.input.read_byte()?;
				let t = Type::try_from(byte)?;
				if matches!(t, Type::MapEnd) {
					Ok(value)
				} else {
					Err(Error::WrongType(t, &[Type::MapEnd]))
				}
			}
			_ => Err(Error::WrongType(t, &[Type::MapStart])),
		}
	}

	fn deserialize_struct<V>(
		self,
		_: &'static str,
		_: &'static [&'static str],
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_enum<V>(
		self,
		name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null => self.deserialize_null(visitor),
			Type::UnsignedInt => {
				_ = self.input.read_byte()?;
				let index = u32::decode(&mut self.input)?;
				visitor.visit_enum(index.into_deserializer())
			}
			Type::String => {
				_ = self.input.read_byte()?;
				let len = usize::decode(&mut self.input)?;
				let bytes = self.read_bytes(len)?;
				let s = str::from_utf8(bytes)?;
				visitor.visit_enum(s.into_deserializer())
			}
			Type::MapStart => {
				_ = self.input.read_byte()?;
				let value = visitor.visit_enum(EnumMapDeserializer(self))?;

				let byte = self.input.read_byte()?;
				let t = Type::try_from(byte)?;
				if matches!(t, Type::MapEnd) {
					Ok(value)
				} else {
					Err(Error::WrongType(t, &[Type::MapEnd]))
				}
			}
			_ => Err(Error::WrongType(
				t,
				&[Type::Null, Type::String, Type::MapStart],
			)),
		}
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let t = self.peek_type()?;
		match t {
			Type::Null | Type::True | Type::False => {
				_ = self.input.read_byte()?;
			}
			Type::UnsignedInt | Type::SignedInt => {
				_ = self.input.read_byte()?;
				while !matches!(self.input.read_byte()? & 0x80, 0) {}
			}
			Type::Float16 => {
				self.input.skip_bytes(3)?;
			}
			Type::Float32 => {
				self.input.skip_bytes(5)?;
			}
			Type::Float64 => {
				self.input.skip_bytes(9)?;
			}
			Type::Float128 => {
				self.input.skip_bytes(17)?;
			}
            Type::Bytes | Type::String => {
                _ = self.input.read_byte()?;
                let len = usize::decode(&mut self.input)?;
                self.input.skip_bytes(len)?;
            }
            Type::SeqStart => return self.deserialize_seq(visitor),
            Type::MapStart => return self.deserialize_map(visitor),
            Type::SeqEnd | Type::MapEnd => {
                return Err(Error::WrongType(t, &[
                    Type::Null, Type::False, Type::True, Type::UnsignedInt, Type::SignedInt, Type::Float16, Type::Float32, Type::Float64, Type::Float128, Type::Bytes
                ]))
            }
		}

		visitor.visit_unit()
	}

	fn is_human_readable(&self) -> bool {
		false
	}
}

#[repr(transparent)]
pub struct MapDeserializer<'a, I, B>(&'a mut Deserializer<I, B>);

impl<'de, I, B: Buffer> MapAccess<'de> for MapDeserializer<'_, I, B>
where
	I: Input<'de>,
{
	type Error = Error;

	fn size_hint(&self) -> Option<usize> {
		None
	}

	fn next_key_seed<K>(&mut self, seed: K) -> core::result::Result<Option<K::Value>, Self::Error>
	where
		K: DeserializeSeed<'de>,
	{
		let byte = self.0.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		if matches!(t, Type::MapEnd) {
			return Ok(None);
		}

		seed.deserialize(&mut *self.0).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		seed.deserialize(&mut *self.0)
	}
}

#[repr(transparent)]
pub struct EnumMapDeserializer<'a, I, B>(&'a mut Deserializer<I, B>);

impl<'de, I, B: Buffer> EnumAccess<'de> for EnumMapDeserializer<'_, I, B>
where
	I: Input<'de>,
{
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(
		self,
		seed: V,
	) -> core::result::Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let value = seed.deserialize(&mut *self.0)?;
		Ok((value, self))
	}
}

impl<'de, I, B: Buffer> VariantAccess<'de> for EnumMapDeserializer<'_, I, B>
where
	I: Input<'de>,
{
	type Error = Error;

	fn unit_variant(self) -> core::result::Result<(), Self::Error> {
		let byte = self.0.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		let found = match t {
			Type::SeqStart => Unexpected::TupleVariant,
			Type::MapStart => Unexpected::StructVariant,
			_ => Unexpected::NewtypeVariant,
		};

		Err(DeError::invalid_type(found, &"unit variant"))
	}

	fn newtype_variant_seed<T>(self, seed: T) -> core::result::Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(self.0)
	}

	fn tuple_variant<V>(self, _: usize, visitor: V) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		SerdeDeserializer::deserialize_seq(self.0, visitor)
	}

	fn struct_variant<V>(
		self,
		_: &'static [&'static str],
		visitor: V,
	) -> core::result::Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		SerdeDeserializer::deserialize_map(self.0, visitor)
	}
}

#[repr(transparent)]
pub struct CharSequenceDeserializer<'a>(str::Chars<'a>);

impl<'de> SeqAccess<'de> for CharSequenceDeserializer<'_> {
	type Error = Error;

	fn size_hint(&self) -> Option<usize> {
		None
	}

	fn next_element_seed<T>(
		&mut self,
		seed: T,
	) -> core::result::Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		if let Some(c) = self.0.next() {
			seed.deserialize(c.into_deserializer()).map(Some)
		} else {
			Ok(None)
		}
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ByteSequenceDeserializer<'a>(&'a [u8]);

impl<'de> SeqAccess<'de> for ByteSequenceDeserializer<'_> {
	type Error = Error;

	fn next_element_seed<T>(
		&mut self,
		seed: T,
	) -> core::result::Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		if let Some((byte, remaining)) = self.0.split_first() {
			self.0 = remaining;
			seed.deserialize(byte.into_deserializer()).map(Some)
		} else {
			Ok(None)
		}
	}

	fn size_hint(&self) -> Option<usize> {
		Some(self.0.len())
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct SequenceDeserializer<'a, I, B>(&'a mut Deserializer<I, B>);

impl<'de, I, B: Buffer> SeqAccess<'de> for SequenceDeserializer<'_, I, B>
where
	I: Input<'de>,
{
	type Error = Error;

	fn size_hint(&self) -> Option<usize> {
		None
	}

	fn next_element_seed<T>(
		&mut self,
		seed: T,
	) -> core::result::Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		let byte = self.0.input.peek_byte()?;
		let t = Type::try_from(byte)?;
		if matches!(t, Type::SeqEnd) {
			return Ok(None);
		}

		seed.deserialize(&mut *self.0).map(Some)
	}
}
