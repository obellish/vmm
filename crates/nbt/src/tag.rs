use std::io::Read;

use super::{
	BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, DOUBLE_ID, END_ID, Error, FLOAT_ID, INT_ARRAY_ID, INT_ID,
	LIST_ID, LONG_ARRAY_ID, LONG_ID, Result, SHORT_ID, STRING_ID, compound::NbtCompound,
	deserializer::NbtReadHelper,
};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NbtTag {
	End = END_ID,
	Byte(i8) = BYTE_ID,
	Short(i16) = SHORT_ID,
	Int(i32) = INT_ID,
	Long(i64) = LONG_ID,
	Float(f32) = FLOAT_ID,
	Double(f64) = DOUBLE_ID,
	ByteArray(Box<[u8]>) = BYTE_ARRAY_ID,
	String(String) = STRING_ID,
	List(Box<[NbtTag]>) = LIST_ID,
	Compound(NbtCompound) = COMPOUND_ID,
	IntArray(Box<[i32]>) = INT_ARRAY_ID,
	LongArray(Box<[i64]>) = LONG_ARRAY_ID,
}

impl NbtTag {
	#[must_use]
	pub const fn get_type_id(&self) -> u8 {
		unsafe { *std::ptr::from_ref::<Self>(self).cast::<u8>() }
	}

	pub fn skip_data<R: Read>(reader: &mut NbtReadHelper<R>, tag_id: u8) -> Result<()> {
		match tag_id {
			END_ID => return Ok(()),
			BYTE_ID => reader.skip_bytes(1)?,
			SHORT_ID => reader.skip_bytes(2)?,
			INT_ID | FLOAT_ID => reader.skip_bytes(4)?,
			LONG_ID | DOUBLE_ID => reader.skip_bytes(8)?,
			BYTE_ARRAY_ID => {
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				reader.skip_bytes(len as u64)?;
			}
			STRING_ID => {
				let len = reader.try_get_u16_be()?;
				reader.skip_bytes(u64::from(len))?;
			}
			LIST_ID => {
				let tag_type_id = reader.try_get_u8_be()?;
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				for _ in 0..len {
					Self::skip_data(reader, tag_type_id)?;
				}
			}
			COMPOUND_ID => todo!(),
			INT_ARRAY_ID => {
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				reader.skip_bytes(len as u64 * 4)?;
			}
			LONG_ARRAY_ID => {
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				reader.skip_bytes(len as u64 * 8)?;
			}
			t => return Err(Error::UnknownTagId(t)),
		}

		Ok(())
	}

	pub fn deserialize_data<R: Read>(
		reader: &mut NbtReadHelper<R>,
		tag_id: u8,
	) -> Result<Self, Error> {
		match tag_id {
			END_ID => Ok(Self::End),
			BYTE_ID => {
				let byte = reader.try_get_i8_be()?;
				Ok(Self::Byte(byte))
			}
			SHORT_ID => {
				let short = reader.try_get_i16_be()?;
				Ok(Self::Short(short))
			}
			INT_ID => {
				let int = reader.try_get_i32_be()?;
				Ok(Self::Int(int))
			}
			LONG_ID => {
				let long = reader.try_get_i64_be()?;
				Ok(Self::Long(long))
			}
			FLOAT_ID => {
				let float = reader.try_get_f32_be()?;
				Ok(Self::Float(float))
			}
			DOUBLE_ID => {
				let double = reader.try_get_f64_be()?;
				Ok(Self::Double(double))
			}
			BYTE_ARRAY_ID => {
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				let byte_array = reader.try_read_boxed_slice(len as usize)?;
				Ok(Self::ByteArray(byte_array))
			}
			STRING_ID => Ok(Self::String(reader.try_get_nbt_string()?)),
			LIST_ID => {
				let tag_type_id = reader.try_get_u8_be()?;
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				let mut list = Vec::with_capacity(len as usize);
				for _ in 0..len {
					let tag = Self::deserialize_data(reader, tag_type_id)?;
					assert_eq!(tag.get_type_id(), tag_type_id);
					list.push(tag);
				}
				Ok(Self::List(list.into_boxed_slice()))
			}
			INT_ARRAY_ID => {
				let len = reader.try_get_i32_be()?;
				if len < 0 {
					return Err(Error::NegativeLength(len));
				}

				let len = len as usize;
				let mut int_array = Vec::with_capacity(len);
				for _ in 0..len {
					let int = reader.try_get_i32_be()?;
					int_array.push(int);
				}
				Ok(Self::IntArray(int_array.into_boxed_slice()))
			}
			t => Err(Error::UnknownTagId(t)),
		}
	}

	#[must_use]
	pub const fn as_i8(&self) -> Option<i8> {
		let Self::Byte(b) = self else {
			return None;
		};

		Some(*b)
	}

	#[must_use]
	pub const fn as_i16(&self) -> Option<i16> {
		let Self::Short(s) = self else {
			return None;
		};

		Some(*s)
	}

	#[must_use]
	pub const fn as_i32(&self) -> Option<i32> {
		let Self::Int(i) = self else {
			return None;
		};

		Some(*i)
	}

	#[must_use]
	pub const fn as_i64(&self) -> Option<i64> {
		let Self::Long(l) = self else {
			return None;
		};

		Some(*l)
	}

	#[must_use]
	pub const fn as_f32(&self) -> Option<f32> {
		let Self::Float(f) = self else {
			return None;
		};

		Some(*f)
	}

	#[must_use]
	pub const fn as_f64(&self) -> Option<f64> {
		let Self::Double(d) = self else {
			return None;
		};

		Some(*d)
	}

	#[must_use]
	pub const fn as_bool(&self) -> Option<bool> {
		let Self::Byte(b) = self else {
			return None;
		};

		Some(!matches!(*b, 0))
	}

	#[must_use]
	pub fn as_bytes(&self) -> Option<&[u8]> {
		let Self::ByteArray(b) = self else {
			return None;
		};

		Some(b)
	}

	#[must_use]
	pub fn as_str(&self) -> Option<&str> {
		let Self::String(s) = self else {
			return None;
		};

		Some(s.as_str())
	}

	#[must_use]
	pub fn as_slice(&self) -> Option<&[Self]> {
		let Self::List(l) = self else {
			return None;
		};

		Some(l)
	}

	#[must_use]
	pub const fn as_compound(&self) -> Option<&NbtCompound> {
		let Self::Compound(c) = self else {
			return None;
		};

		Some(c)
	}

	#[must_use]
	pub fn as_i32_slice(&self) -> Option<&[i32]> {
		let Self::IntArray(i) = self else {
			return None;
		};

		Some(i)
	}

	#[must_use]
	pub fn as_i64_slice(&self) -> Option<&[i64]> {
		let Self::LongArray(l) = self else {
			return None;
		};

		Some(l)
	}
}

impl From<f32> for NbtTag {
	fn from(value: f32) -> Self {
		Self::Float(value)
	}
}

impl From<f64> for NbtTag {
	fn from(value: f64) -> Self {
		Self::Double(value)
	}
}

impl From<bool> for NbtTag {
	fn from(value: bool) -> Self {
		Self::Byte(i8::from(value))
	}
}

impl From<String> for NbtTag {
	fn from(value: String) -> Self {
		Self::String(value)
	}
}

impl FromIterator<u8> for NbtTag {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = u8>,
	{
		Self::ByteArray(iter.into_iter().collect())
	}
}
