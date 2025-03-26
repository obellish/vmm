use std::{
	borrow::Cow,
	hash::Hash,
	io::{Result as IoResult, prelude::*},
};

use byteorder::{BigEndian, WriteBytesExt as _};

use super::modified_utf8;
use crate::{Compound, Error, List, Result, Tag, Value, conv::i8_slice_as_u8_slice};

#[repr(transparent)]
struct EncodeState<W> {
	writer: W,
}

impl<W: Write> EncodeState<W> {
	fn write_tag(&mut self, tag: Tag) -> IoResult<()> {
		self.writer.write_u8(tag as u8)
	}

	fn write_value<S>(&mut self, v: &Value<S>) -> Result<()>
	where
		S: Hash + Ord + ToModifiedUtf8,
	{
		match v {
			Value::Byte(v) => self.write_byte(*v).map_err(Into::into),
			Value::Short(v) => self.write_short(*v).map_err(Into::into),
			Value::Int(v) => self.write_int(*v).map_err(Into::into),
			Value::Long(v) => self.write_long(*v).map_err(Into::into),
			Value::Float(v) => self.write_float(*v).map_err(Into::into),
			Value::Double(v) => self.write_double(*v).map_err(Into::into),
			Value::ByteArray(v) => self.write_byte_array(v),
			Value::String(v) => self.write_string(v),
			Value::List(v) => self.write_any_list(v),
			Value::Compound(v) => self.write_compound(v),
			Value::IntArray(v) => self.write_int_array(v),
			Value::LongArray(v) => self.write_long_array(v),
		}
	}

	fn write_byte(&mut self, byte: i8) -> IoResult<()> {
		self.writer.write_i8(byte)
	}

	fn write_short(&mut self, short: i16) -> IoResult<()> {
		self.writer.write_i16::<BigEndian>(short)
	}

	fn write_int(&mut self, int: i32) -> IoResult<()> {
		self.writer.write_i32::<BigEndian>(int)
	}

	fn write_long(&mut self, long: i64) -> IoResult<()> {
		self.writer.write_i64::<BigEndian>(long)
	}

	fn write_float(&mut self, float: f32) -> IoResult<()> {
		self.writer.write_f32::<BigEndian>(float)
	}

	fn write_double(&mut self, double: f64) -> IoResult<()> {
		self.writer.write_f64::<BigEndian>(double)
	}

	fn write_byte_array(&mut self, bytes: &[i8]) -> Result<()> {
		match bytes.len().try_into() {
			Ok(len) => self.write_int(len)?,
			Err(_) => {
				return Err(Error::owned(format!(
					"byte array of length {} exceeds maximum of i32::MAX",
					bytes.len()
				)));
			}
		}

		Ok(self.writer.write_all(i8_slice_as_u8_slice(bytes))?)
	}

	fn write_string<S>(&mut self, s: &S) -> Result<()>
	where
		S: ?Sized + ToModifiedUtf8,
	{
		let len = s.modified_utf8_len();

		match len.try_into() {
			Ok(n) => self.writer.write_u16::<BigEndian>(n)?,
			Err(_) => {
				return Err(Error::owned(format!(
					"string of length {len} exceeds maximum of u16::MAX"
				)));
			}
		}

		s.to_modified_utf8(len, &mut self.writer)?;

		Ok(())
	}

	fn write_any_list<S>(&mut self, list: &List<S>) -> Result<()>
	where
		S: Hash + Ord + ToModifiedUtf8,
	{
		match list {
			List::End => {
				self.write_tag(Tag::End)?;
				// Length
				self.writer.write_i32::<BigEndian>(0)?;
				Ok(())
			}
			List::Byte(v) => {
				self.write_tag(Tag::Byte)?;

				match v.len().try_into() {
					Ok(len) => self.write_int(len)?,
					Err(_) => {
						return Err(Error::owned(format!(
							"byte list of length {} exceeds maximum of i32::MAX",
							v.len(),
						)));
					}
				}

				Ok(self.writer.write_all(i8_slice_as_u8_slice(v))?)
			}
			List::Short(sl) => self.write_list(sl, Tag::Short, |st, v| {
				st.write_short(*v).map_err(Into::into)
			}),
			List::Int(il) => {
				self.write_list(il, Tag::Int, |st, v| st.write_int(*v).map_err(Into::into))
			}
			List::Long(ll) => {
				self.write_list(ll, Tag::Long, |st, v| st.write_long(*v).map_err(Into::into))
			}
			List::Float(fl) => self.write_list(fl, Tag::Float, |st, v| {
				st.write_float(*v).map_err(Into::into)
			}),
			List::Double(dl) => self.write_list(dl, Tag::Double, |st, v| {
				st.write_double(*v).map_err(Into::into)
			}),
			List::ByteArray(v) => {
				self.write_list(v, Tag::ByteArray, |st, v| st.write_byte_array(v))
			}
			List::String(v) => self.write_list(v, Tag::String, Self::write_string),
			List::List(v) => self.write_list(v, Tag::List, Self::write_any_list),
			List::Compound(v) => self.write_list(v, Tag::Compound, Self::write_compound),
			List::IntArray(v) => self.write_list(v, Tag::IntArray, |st, v| st.write_int_array(v)),
			List::LongArray(v) => {
				self.write_list(v, Tag::LongArray, |st, v| st.write_long_array(v))
			}
		}
	}

	fn write_list<T>(
		&mut self,
		list: &[T],
		elem_type: Tag,
		mut write_elem: impl FnMut(&mut Self, &T) -> Result<()>,
	) -> Result<()> {
		self.write_tag(elem_type)?;

		match list.len().try_into() {
			Ok(len) => self.write_int(len)?,
			Err(_) => {
				return Err(Error::owned(format!(
					"{} list of length {} exceeds maximum of i32::MAX",
					list.len(),
					elem_type.name()
				)));
			}
		}

		for elem in list {
			write_elem(self, elem)?;
		}

		Ok(())
	}

	fn write_compound<S>(&mut self, c: &Compound<S>) -> Result<()>
	where
		S: Hash + Ord + ToModifiedUtf8,
	{
		for (k, v) in c {
			self.write_tag(v.tag())?;
			self.write_string(k)?;
			self.write_value(v)?;
		}

		self.write_tag(Tag::End)?;

		Ok(())
	}

	fn write_int_array(&mut self, ia: &[i32]) -> Result<()> {
		match ia.len().try_into() {
			Ok(len) => self.write_int(len)?,
			Err(_) => {
				return Err(Error::owned(format!(
					"int array of length {} exceeds maximum of i32::MAX",
					ia.len()
				)));
			}
		}

		for i in ia {
			self.write_int(*i)?;
		}

		Ok(())
	}

	fn write_long_array(&mut self, la: &[i64]) -> Result<()> {
		match la.len().try_into() {
			Ok(len) => self.write_int(len)?,
			Err(_) => {
				return Err(Error::owned(format!(
					"long array of length {} exceeds maximum of i32::MAX",
					la.len()
				)));
			}
		}

		for l in la {
			self.write_long(*l)?;
		}

		Ok(())
	}
}

pub trait ToModifiedUtf8 {
	fn modified_utf8_len(&self) -> usize;
	fn to_modified_utf8(&self, encoded_len: usize, writer: impl Write) -> IoResult<()>;
}

impl ToModifiedUtf8 for str {
	fn modified_utf8_len(&self) -> usize {
		modified_utf8::encoded_len(self.as_bytes())
	}

	fn to_modified_utf8(&self, encoded_len: usize, mut writer: impl Write) -> IoResult<()> {
		if self.len() == encoded_len {
			writer.write_all(self.as_bytes())
		} else {
			modified_utf8::write_modified_utf8(writer, self)
		}
	}
}

impl ToModifiedUtf8 for Cow<'_, str> {
	fn modified_utf8_len(&self) -> usize {
		str::modified_utf8_len(self)
	}

	fn to_modified_utf8(&self, encoded_len: usize, writer: impl Write) -> IoResult<()> {
		str::to_modified_utf8(self, encoded_len, writer)
	}
}

impl ToModifiedUtf8 for String {
	fn modified_utf8_len(&self) -> usize {
		str::modified_utf8_len(self)
	}

	fn to_modified_utf8(&self, encoded_len: usize, writer: impl Write) -> IoResult<()> {
		str::to_modified_utf8(self, encoded_len, writer)
	}
}

#[cfg(feature = "java_string")]
impl ToModifiedUtf8 for java_string::JavaStr {
	fn modified_utf8_len(&self) -> usize {
		modified_utf8::encoded_len(self.as_bytes())
	}

	fn to_modified_utf8(&self, _: usize, mut writer: impl Write) -> IoResult<()> {
		writer.write_all(&self.to_modified_utf8())
	}
}

#[cfg(feature = "java_string")]
impl ToModifiedUtf8 for Cow<'_, java_string::JavaStr> {
	fn modified_utf8_len(&self) -> usize {
		java_string::JavaStr::modified_utf8_len(self)
	}

	fn to_modified_utf8(&self, encoded_len: usize, writer: impl Write) -> IoResult<()> {
		<java_string::JavaStr as ToModifiedUtf8>::to_modified_utf8(self, encoded_len, writer)
	}
}

#[cfg(feature = "java_string")]
impl ToModifiedUtf8 for java_string::JavaString {
	fn modified_utf8_len(&self) -> usize {
		java_string::JavaStr::modified_utf8_len(self)
	}

	fn to_modified_utf8(&self, encoded_len: usize, writer: impl Write) -> IoResult<()> {
		<java_string::JavaStr as ToModifiedUtf8>::to_modified_utf8(self, encoded_len, writer)
	}
}

pub fn to_binary<S, R>(comp: &Compound<S>, writer: impl Write, root_name: &R) -> Result<()>
where
	S: Hash + Ord + ToModifiedUtf8,
	R: ?Sized + ToModifiedUtf8,
{
	let mut state = EncodeState { writer };

	state.write_tag(Tag::Compound)?;
	state.write_string(root_name)?;
	state.write_compound(comp)?;

	Ok(())
}

pub fn written_size<S, R>(comp: &Compound<S>, root_name: &R) -> usize
where
	S: Hash + Ord + ToModifiedUtf8,
	R: ?Sized + ToModifiedUtf8,
{
	1 + string_size(root_name) + compound_size(comp)
}

fn value_size<S>(val: &Value<S>) -> usize
where
	S: Hash + Ord + ToModifiedUtf8,
{
	match val {
		Value::Byte(_) => 1,
		Value::Short(_) => 2,
		Value::Int(_) | Value::Float(_) => 4,
		Value::Long(_) | Value::Double(_) => 8,
		Value::ByteArray(v) => 4 + v.len(),
		Value::String(s) => string_size(s),
		Value::List(l) => list_size(l),
		Value::Compound(c) => compound_size(c),
		Value::IntArray(v) => 4 + v.len() * 4,
		Value::LongArray(v) => 4 + v.len() * 8,
	}
}

fn list_size<S>(l: &List<S>) -> usize
where
	S: Hash + Ord + ToModifiedUtf8,
{
	let elems_size = match l {
		List::End => 0,
		List::Byte(v) => v.len(),
		List::Short(v) => v.len() * 2,
		List::Int(v) => v.len() * 4,
		List::Long(v) => v.len() * 8,
		List::Float(v) => v.len() * 4,
		List::Double(v) => v.len() * 8,
		List::ByteArray(v) => v.iter().map(|b| 4 + b.len()).sum(),
		List::String(v) => v.iter().map(|s| string_size(s)).sum(),
		List::List(v) => v.iter().map(list_size).sum(),
		List::Compound(v) => v.iter().map(compound_size).sum(),
		List::IntArray(v) => v.iter().map(|i| 4 + i.len() * 4).sum(),
		List::LongArray(v) => v.iter().map(|l| 4 + l.len() * 8).sum(),
	};

	1 + 4 + elems_size
}

fn string_size<S>(s: &S) -> usize
where
	S: ?Sized + ToModifiedUtf8,
{
	2 + s.modified_utf8_len()
}

fn compound_size<S>(c: &Compound<S>) -> usize
where
	S: Hash + Ord + ToModifiedUtf8,
{
	c.iter()
		.map(|(k, v)| 1 + string_size(k) + value_size(v))
		.sum::<usize>()
		+ 1
}
