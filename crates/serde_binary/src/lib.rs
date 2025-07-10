#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buffer;
mod config;
pub mod de;
mod error;
pub mod format;
mod io;
pub mod ser;
#[cfg(test)]
mod tests;
#[cfg(feature = "alloc")]
pub mod value;

#[cfg(feature = "std")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
pub use self::value::{Value, from_value, from_value_with_config, to_value, to_value_with_config};
pub use self::{buffer::*, config::*, error::*, format::Type, io::*};

pub fn to_slice<'buf, T: Serialize>(value: &T, buffer: &'buf mut [u8]) -> Result<&'buf mut [u8]> {
	to_slice_with_config(value, buffer, Config::default())
}

pub fn to_slice_with_config<'buf, T: Serialize>(
	value: &T,
	buffer: &'buf mut [u8],
	config: Config,
) -> Result<&'buf mut [u8]> {
	let remaining = if let Some(max) = config.max_size {
		let mut ser = self::ser::Serializer::new(SizeLimit::new(&mut *buffer, max.get()))
			.use_indices(config.use_indices);

		value.serialize(&mut ser)?;
		ser.into_output().into_inner().len()
	} else {
		let mut ser = self::ser::Serializer::new(&mut *buffer).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		ser.into_output().len()
	};

	let used = buffer.len() - remaining;
	Ok(buffer.split_at_mut(used).0)
}

#[cfg(feature = "alloc")]
pub fn to_vec_with_config<T: Serialize>(value: &T, config: Config) -> Result<alloc::vec::Vec<u8>> {
	if let Some(max) = config.max_size {
		let mut ser = self::ser::Serializer::new(SizeLimit::new(alloc::vec::Vec::new(), max.get()))
			.use_indices(config.use_indices);

		value.serialize(&mut ser)?;
		Ok(ser.into_output().into_inner())
	} else {
		let mut ser =
			self::ser::Serializer::new(alloc::vec::Vec::new()).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
		Ok(ser.into_output())
	}
}

#[cfg(feature = "alloc")]
pub fn to_vec<T: Serialize>(value: &T) -> Result<alloc::vec::Vec<u8>> {
	to_vec_with_config(value, Config::default())
}

#[cfg(feature = "std")]
pub fn to_writer_with_config<T: Serialize, W: std::io::Write>(
	value: &T,
	writer: W,
	config: Config,
) -> Result<()> {
	if let Some(max) = config.max_size {
		let mut ser = self::ser::Serializer::new(SizeLimit::new(IoWriter::new(writer), max.get()))
			.use_indices(config.use_indices);

		value.serialize(&mut ser)?;
	} else {
		let mut ser =
			self::ser::Serializer::new(IoWriter::new(writer)).use_indices(config.use_indices);
		value.serialize(&mut ser)?;
	}

	Ok(())
}

#[cfg(feature = "std")]
pub fn to_writer<T: Serialize, W: std::io::Write>(value: &T, writer: W) -> Result<()> {
	to_writer_with_config(value, writer, Config::default())
}

pub fn from_slice_with_config<'de, T>(bytes: &'de [u8], config: Config) -> Result<T>
where
	T: Deserialize<'de>,
{
	let error_on_excess = config.error_on_excess_data;

	let (value, peek) = if let Some(max) = config.max_size {
		let mut de = self::de::Deserializer::new(SizeLimit::new(bytes, max.get()));
		(
			T::deserialize(&mut de)?,
			Input::peek_byte(&mut de.into_input()),
		)
	} else {
		let mut de = self::de::Deserializer::new(bytes);
		(
			T::deserialize(&mut de)?,
			Input::peek_byte(&mut de.into_input()),
		)
	};

	if error_on_excess && peek.is_ok() {
		Err(Error::ExcessData)
	} else {
		Ok(value)
	}
}

pub fn from_slice<'de, T>(bytes: &'de [u8]) -> Result<T>
where
	T: Deserialize<'de>,
{
	from_slice_with_config(bytes, Config::default())
}

#[cfg(feature = "std")]
pub fn from_reader_with_config<R: std::io::Read, T>(reader: R, config: Config) -> Result<T>
where
	T: DeserializeOwned,
{
	let error_on_excess = config.error_on_excess_data;

	let (value, peek) = if let Some(max) = config.max_size {
		let mut de = self::de::Deserializer::new(SizeLimit::new(IoReader::new(reader), max.get()))
			.and_with_buffer(std::vec::Vec::new());
		(
			T::deserialize(&mut de)?,
			Input::peek_byte(&mut de.into_input()),
		)
	} else {
		let mut de = self::de::Deserializer::new(IoReader::new(reader))
			.and_with_buffer(std::vec::Vec::new());
		(
			T::deserialize(&mut de)?,
			Input::peek_byte(&mut de.into_input()),
		)
	};

	if error_on_excess && peek.is_ok() {
		Err(Error::ExcessData)
	} else {
		Ok(value)
	}
}

#[cfg(feature = "std")]
pub fn from_reader<R: std::io::Read, T>(reader: R) -> Result<T>
where
	T: DeserializeOwned,
{
	from_reader_with_config(reader, Config::default())
}
