use std::io::{self, Error as IoError, Read};

use serde::{
	Deserialize,
	de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor},
	forward_to_deserialize_any,
};

use super::Error;

#[derive(Debug)]
#[repr(transparent)]
pub struct NbtReadHelper<R> {
	reader: R,
}

impl<R> NbtReadHelper<R> {
	pub const fn new(reader: R) -> Self {
		Self { reader }
	}
}

impl<R: Read> NbtReadHelper<R> {
	pub fn skip_bytes(&mut self, count: u64) -> Result<(), IoError> {
		let _ = io::copy(&mut self.reader.by_ref().take(count), &mut io::sink())?;

		Ok(())
	}

	pub fn try_get_nbt_string(&mut self) -> Result<String, super::Error> {
		let len = self.try_get_u16_be()? as usize;
		let string_bytes = self.try_read_boxed_slice(len)?;
		let string = cesu8::from_java_cesu8(&string_bytes)?;
		Ok(string.into_owned())
	}

	pub fn try_get_u8_be(&mut self) -> Result<u8, IoError> {
		let mut buf = [0u8];
		self.reader.read_exact(&mut buf)?;

		Ok(u8::from_be_bytes(buf))
	}

	pub fn try_get_i8_be(&mut self) -> Result<i8, IoError> {
		let mut buf = [0];
		self.reader.read_exact(&mut buf)?;

		Ok(i8::from_be_bytes(buf))
	}

	pub fn try_get_i16_be(&mut self) -> Result<i16, IoError> {
		let mut buf = [0; 2];
		self.reader.read_exact(&mut buf)?;

		Ok(i16::from_be_bytes(buf))
	}

	pub fn try_get_u16_be(&mut self) -> Result<u16, IoError> {
		let mut buf = [0; 2];
		self.reader.read_exact(&mut buf)?;

		Ok(u16::from_be_bytes(buf))
	}

	pub fn try_get_i32_be(&mut self) -> Result<i32, IoError> {
		let mut buf = [0; 4];
		self.reader.read_exact(&mut buf)?;

		Ok(i32::from_be_bytes(buf))
	}

	pub fn try_get_i64_be(&mut self) -> Result<i64, IoError> {
		let mut buf = [0; 8];
		self.reader.read_exact(&mut buf)?;

		Ok(i64::from_be_bytes(buf))
	}

	pub fn try_get_f32_be(&mut self) -> Result<f32, IoError> {
		let mut buf = [0; 4];
		self.reader.read_exact(&mut buf)?;

		Ok(f32::from_be_bytes(buf))
	}

	pub fn try_get_f64_be(&mut self) -> Result<f64, IoError> {
		let mut buf = [0; 8];
		self.reader.read_exact(&mut buf)?;

		Ok(f64::from_be_bytes(buf))
	}

	pub fn try_read_boxed_slice(&mut self, count: usize) -> Result<Box<[u8]>, IoError> {
		let mut buf = vec![0u8; count];
		self.reader.read_exact(&mut buf)?;

		Ok(buf.into_boxed_slice())
	}
}

#[derive(Debug)]
pub struct Deserializer<R> {
	input: NbtReadHelper<R>,
	tag_to_deserialize_stack: Vec<u8>,
	in_list: bool,
	is_named: bool,
}

impl<R> Deserializer<R> {
	pub const fn new(input: R, is_named: bool) -> Self {
		Self {
			input: NbtReadHelper::new(input),
			tag_to_deserialize_stack: Vec::new(),
			in_list: false,
			is_named,
		}
	}
}
