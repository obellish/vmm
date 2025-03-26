use std::{
	borrow::Cow,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	hash::Hash,
	io::Result as IoResult,
	mem,
};

use byteorder::{BigEndian, ReadBytesExt as _};

use crate::{Compound, Error, List, Result, Tag, Value};

const MAX_DEPTH: usize = 512;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct FromModifiedUtf8Error;

impl Display for FromModifiedUtf8Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("could not decode modified UTF-8 data")
	}
}

impl StdError for FromModifiedUtf8Error {}

struct DecodeState<'a, 'de> {
	slice: &'a mut &'de [u8],
	depth: usize,
}

impl<'de> DecodeState<'_, 'de> {
	fn check_depth<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
		if self.depth >= MAX_DEPTH {
			return Err(Error::r#static("reached maximum recursion depth"));
		}

		self.depth += 1;
		let res = f(self);
		self.depth -= 1;
		res
	}

	fn read_byte(&mut self) -> IoResult<i8> {
		self.slice.read_i8()
	}

	fn read_short(&mut self) -> IoResult<i16> {
		self.slice.read_i16::<BigEndian>()
	}

	fn read_int(&mut self) -> IoResult<i32> {
		self.slice.read_i32::<BigEndian>()
	}

	fn read_long(&mut self) -> IoResult<i64> {
		self.slice.read_i64::<BigEndian>()
	}

	fn read_float(&mut self) -> IoResult<f32> {
		self.slice.read_f32::<BigEndian>()
	}

	fn read_double(&mut self) -> IoResult<f64> {
		self.slice.read_f64::<BigEndian>()
	}

	fn read_byte_array(&mut self) -> Result<Vec<i8>> {
		let len = self.read_int()?;

		if len.is_negative() {
			return Err(Error::owned(format!("negative byte array length of {len}")));
		}

		if len as usize > self.slice.len() {
			return Err(Error::owned(format!(
				"byte array length of {len} exceeds remainder of input"
			)));
		}

		let (left, right) = self.slice.split_at(len as usize);

		let array = left.iter().map(|b| *b as i8).collect();
		*self.slice = right;

		Ok(array)
	}

	fn read_string<S>(&mut self) -> Result<S>
	where
		S: FromModifiedUtf8<'de>,
	{
		let len = self.slice.read_u16::<BigEndian>()?.into();

		if len > self.slice.len() {
			return Err(Error::owned(format!(
				"string of length {len} exceeds remainder of input"
			)));
		}

		let (left, right) = self.slice.split_at(len);

		match S::from_modified_utf8(left) {
			Ok(str) => {
				*self.slice = right;
				Ok(str)
			}
			Err(_) => Err(Error::r#static("could not decode modified UTF-8 data")),
		}
	}

	fn read_tag(&mut self) -> Result<Tag> {
		match self.slice.read_u8()? {
			0 => Ok(Tag::End),
			1 => Ok(Tag::Byte),
			2 => Ok(Tag::Short),
			3 => Ok(Tag::Int),
			4 => Ok(Tag::Long),
			5 => Ok(Tag::Float),
			6 => Ok(Tag::Double),
			7 => Ok(Tag::ByteArray),
			8 => Ok(Tag::String),
			9 => Ok(Tag::List),
			10 => Ok(Tag::Compound),
			11 => Ok(Tag::IntArray),
			12 => Ok(Tag::LongArray),
			byte => Err(Error::owned(format!("invalid tag byte of {byte:#x}"))),
		}
	}

	fn read_value<S>(&mut self, tag: Tag) -> Result<Value<S>>
	where
		S: FromModifiedUtf8<'de> + Hash + Ord,
	{
		match tag {
			Tag::End => unreachable!("illegal TAG_End argument"),
			Tag::Byte => Ok(self.read_byte()?.into()),
			Tag::Short => Ok(self.read_short()?.into()),
			Tag::Int => Ok(self.read_int()?.into()),
			Tag::Long => Ok(self.read_long()?.into()),
			Tag::Float => Ok(self.read_float()?.into()),
			Tag::Double => Ok(self.read_double()?.into()),
			Tag::ByteArray => Ok(self.read_byte_array()?.into()),
			Tag::String => Ok(Value::String(self.read_string::<S>()?)),
			Tag::List => self.check_depth(|st| Ok(st.read_any_list::<S>()?.into())),
			Tag::Compound => self.check_depth(|st| Ok(st.read_compound::<S>()?.into())),
			Tag::IntArray => Ok(self.read_int_array()?.into()),
			Tag::LongArray => Ok(self.read_long_array()?.into()),
		}
	}

	fn read_any_list<S>(&mut self) -> Result<List<S>>
	where
		S: FromModifiedUtf8<'de> + Hash + Ord,
	{
		match self.read_tag()? {
			Tag::End => match self.read_int()? {
				0 => Ok(List::End),
				len => Err(Error::owned(format!(
					"TAG_End list with nonzero length of {len}"
				))),
			},
			Tag::Byte => Ok(self
				.read_list(Tag::Byte, 1, |st| st.read_byte().map_err(Into::into))?
				.into()),
			Tag::Short => Ok(self
				.read_list(Tag::Short, 2, |st| st.read_short().map_err(Into::into))?
				.into()),
			Tag::Int => Ok(self
				.read_list(Tag::Int, 4, |st| st.read_int().map_err(Into::into))?
				.into()),
			Tag::Long => Ok(self
				.read_list(Tag::Long, 8, |st| st.read_long().map_err(Into::into))?
				.into()),
			Tag::Float => Ok(self
				.read_list(Tag::Float, 4, |st| st.read_float().map_err(Into::into))?
				.into()),
			Tag::Double => Ok(self
				.read_list(Tag::Double, 8, |st| st.read_double().map_err(Into::into))?
				.into()),
			Tag::ByteArray => Ok(self
				.read_list(Tag::ByteArray, 0, Self::read_byte_array)?
				.into()),
			Tag::String => Ok(List::String(self.read_list(
				Tag::String,
				0,
				Self::read_string::<S>,
			)?)),
			Tag::List => self
				.check_depth(|st| Ok(st.read_list(Tag::List, 0, Self::read_any_list::<S>)?.into())),
			Tag::Compound => self.check_depth(|st| {
				Ok(st
					.read_list(Tag::Compound, 0, Self::read_compound::<S>)?
					.into())
			}),
			Tag::IntArray => Ok(self
				.read_list(Tag::IntArray, 0, Self::read_int_array)?
				.into()),
			Tag::LongArray => Ok(self
				.read_list(Tag::LongArray, 0, Self::read_long_array)?
				.into()),
		}
	}

	fn read_list<T>(
		&mut self,
		elem_type: Tag,
		elem_size: usize,
		mut read_elem: impl FnMut(&mut Self) -> Result<T>,
	) -> Result<Vec<T>> {
		let len = self.read_int()?;

		if len.is_negative() {
			return Err(Error::owned(format!(
				"negative {} list length of {len}",
				elem_type.name()
			)));
		}

		if len as u64 * elem_size as u64 > self.slice.len() as u64 {
			return Err(Error::owned(format!(
				"{} list of length {len} exceeds remainder of input",
				elem_type.name()
			)));
		}

		let mut list = Vec::with_capacity(if matches!(elem_size, 0) {
			0
		} else {
			len as usize
		});
		for _ in 0..len {
			list.push(read_elem(self)?);
		}

		Ok(list)
	}

	fn read_compound<S>(&mut self) -> Result<Compound<S>>
	where
		S: FromModifiedUtf8<'de> + Hash + Ord,
	{
		let mut compound = Compound::new();

		loop {
			let tag = self.read_tag()?;
			if matches!(tag, Tag::End) {
				break Ok(compound);
			}

			compound.insert(self.read_string::<S>()?, self.read_value::<S>(tag)?);
		}
	}

	fn read_int_array(&mut self) -> Result<Vec<i32>> {
		let len = self.read_int()?;

		if len.is_negative() {
			return Err(Error::owned(format!("negative int array length of {len}")));
		}

		if len as u64 * mem::size_of::<i32>() as u64 > self.slice.len() as u64 {
			return Err(Error::owned(format!(
				"int array of length {len} exceeds remainder of input"
			)));
		}

		let mut array = Vec::with_capacity(len as usize);
		for _ in 0..len {
			array.push(self.read_int()?);
		}

		Ok(array)
	}

	fn read_long_array(&mut self) -> Result<Vec<i64>> {
		let len = self.read_int()?;

		if len.is_negative() {
			return Err(Error::owned(format!("negative long array length of {len}")));
		}

		if len as u64 * mem::size_of::<i64>() as u64 > self.slice.len() as u64 {
			return Err(Error::owned(format!(
				"long array of length {len} exceeds remainder of input"
			)));
		}

		let mut array = Vec::with_capacity(len as usize);
		for _ in 0..len {
			array.push(self.read_long()?);
		}

		Ok(array)
	}
}

pub trait FromModifiedUtf8<'de>: Sized {
	fn from_modified_utf8(modified_utf8: &'de [u8]) -> Result<Self, FromModifiedUtf8Error>;
}

impl<'de> FromModifiedUtf8<'de> for Cow<'de, str> {
	fn from_modified_utf8(modified_utf8: &'de [u8]) -> Result<Self, FromModifiedUtf8Error> {
		cesu8::from_java_cesu8(modified_utf8).map_err(move |_| FromModifiedUtf8Error)
	}
}

impl<'de> FromModifiedUtf8<'de> for String {
	fn from_modified_utf8(modified_utf8: &'de [u8]) -> Result<Self, FromModifiedUtf8Error> {
		match cesu8::from_java_cesu8(modified_utf8) {
			Ok(str) => Ok(str.into_owned()),
			Err(_) => Err(FromModifiedUtf8Error),
		}
	}
}

#[cfg(feature = "java_string")]
impl<'de> FromModifiedUtf8<'de> for Cow<'de, java_string::JavaStr> {
	fn from_modified_utf8(modified_utf8: &'de [u8]) -> Result<Self, FromModifiedUtf8Error> {
		java_string::JavaStr::from_modified_utf8(modified_utf8).map_err(|_| FromModifiedUtf8Error)
	}
}

#[cfg(feature = "java_string")]
impl<'de> FromModifiedUtf8<'de> for java_string::JavaString {
	fn from_modified_utf8(modified_utf8: &'de [u8]) -> Result<Self, FromModifiedUtf8Error> {
		match java_string::JavaStr::from_modified_utf8(modified_utf8) {
			Ok(str) => Ok(str.into_owned()),
			Err(_) => Err(FromModifiedUtf8Error),
		}
	}
}

pub fn from_binary<'de, S>(slice: &mut &'de [u8]) -> Result<(Compound<S>, S)>
where
	S: FromModifiedUtf8<'de> + Hash + Ord,
{
	let mut state = DecodeState { slice, depth: 0 };

	let root_tag = state.read_tag()?;

	if !matches!(root_tag, Tag::Compound) {
		return Err(Error::owned(format!(
			"expected root tag for compound (got {})",
			root_tag.name()
		)));
	}

	let root_name = state.read_string::<S>()?;
	let root = state.read_compound()?;

	debug_assert_eq!(state.depth, 0);

	Ok((root, root_name))
}
