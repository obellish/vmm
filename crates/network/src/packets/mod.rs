#[path = "client.rs"]
pub mod clientbound;
#[path = "server.rs"]
pub mod serverbound;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{self, Cursor, Read, Write},
	net::TcpStream,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
};

use byteorder::{BigEndian, ReadBytesExt as _, WriteBytesExt as _};
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};
use vmm_text::TextComponent;

use super::{NbtCompound, NetworkState};

pub const COMPRESSION_THRESHOLD: usize = 256;

#[derive(Debug)]
pub struct SlotData {
	pub item_id: i32,
	pub item_count: i8,
	pub nbt: Option<NbtCompound>,
}

#[derive(Debug, Clone)]
pub struct PlayerProperty {
	pub name: String,
	pub value: String,
	pub signature: Option<String>,
}

#[derive(Debug)]
pub struct PalettedContainer {
	pub bits_per_entry: u8,
	pub palette: Option<Vec<i32>>,
	pub data_array: Vec<u64>,
}

pub struct PacketEncoder {
	buffer: Vec<u8>,
	packet_id: u32,
}

impl PacketEncoder {}

#[derive(Debug)]
pub enum PacketDecodeError {
	Io(io::Error),
	FromUtf8(std::string::FromUtf8Error),
	Nbt(nbt::Error),
}

impl Display for PacketDecodeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
			Self::FromUtf8(e) => Display::fmt(&e, f),
			Self::Nbt(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for PacketDecodeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::FromUtf8(e) => Some(e),
			Self::Nbt(e) => Some(e),
		}
	}
}

impl From<io::Error> for PacketDecodeError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<std::string::FromUtf8Error> for PacketDecodeError {
	fn from(value: std::string::FromUtf8Error) -> Self {
		Self::FromUtf8(value)
	}
}

impl From<nbt::Error> for PacketDecodeError {
	fn from(value: nbt::Error) -> Self {
		Self::Nbt(value)
	}
}

#[derive(Debug)]
pub enum PacketEncodeError {}

impl Display for PacketEncodeError {
	#[allow(clippy::uninhabited_references)]
	fn fmt(&self, _: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for PacketEncodeError {}

pub trait PacketDecoderExt: Read + Sized {
	fn read_unsigned_byte(&mut self) -> DecodeResult<u8> {
		Ok(self.read_u8()?)
	}

	fn read_byte(&mut self) -> DecodeResult<i8> {
		Ok(self.read_i8()?)
	}

	fn read_bytes(&mut self, bytes: usize) -> DecodeResult<Vec<u8>> {
		let mut read = vec![0; bytes];
		self.read_exact(&mut read)?;
		Ok(read)
	}

	fn read_long(&mut self) -> DecodeResult<i64> {
		Ok(self.read_i64::<BigEndian>()?)
	}

	fn read_int(&mut self) -> DecodeResult<i32> {
		Ok(self.read_i32::<BigEndian>()?)
	}

	fn read_short(&mut self) -> DecodeResult<i16> {
		Ok(self.read_i16::<BigEndian>()?)
	}

	fn read_unsigned_short(&mut self) -> DecodeResult<u16> {
		Ok(self.read_u16::<BigEndian>()?)
	}

	fn read_double(&mut self) -> DecodeResult<f64> {
		Ok(self.read_f64::<BigEndian>()?)
	}

	fn read_float(&mut self) -> DecodeResult<f32> {
		Ok(self.read_f32::<BigEndian>()?)
	}

	fn read_bool(&mut self) -> DecodeResult<bool> {
		// Ok(self.read_u8()? == 1)
		Ok(matches!(self.read_u8()?, 1))
	}
}

pub type DecodeResult<T, E = PacketDecodeError> = std::result::Result<T, E>;
