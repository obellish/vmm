#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

extern crate self as vmm_protocol;

mod array;
mod bounded;
mod difficulty;
mod impls;
mod raw;
pub mod sound;
pub mod var_int;
mod velocity;

#[doc(hidden)]
pub mod __private {
	pub use super::{Decode, Encode, Packet, ProtocolError, VarInt};
}

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::{Error as IoError, Write},
	str::Utf8Error,
};

use serde::{Deserialize, Serialize};
use vmm_ident::IdentError;
use vmm_nbt::Error as NbtError;
pub use vmm_protocol_macros::{Decode, Encode, Packet};

use self::var_int::VarIntDecodeError;
#[doc(inline)]
pub use self::{bounded::Bounded, raw::RawBytes, sound::Sound, var_int::VarInt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CompressionThreshold(pub i32);

impl CompressionThreshold {
	pub const DEFAULT: Self = Self(-1);
}

impl Default for CompressionThreshold {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl From<i32> for CompressionThreshold {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<CompressionThreshold> for i32 {
	fn from(value: CompressionThreshold) -> Self {
		value.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketSide {
	Clientbound,
	Serverbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketState {
	Handshaking,
	Status,
	Login,
	Play,
}

#[derive(Debug)]
pub enum ProtocolError {
	Io(IoError),
	VarIntDecode(VarIntDecodeError),
	FailedToEncodePacketId,
	Nbt(NbtError),
	Ident(IdentError),
	Utf8(Utf8Error),
	Other(String),
}

impl Display for ProtocolError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
			Self::VarIntDecode(e) => Display::fmt(&e, f),
			Self::FailedToEncodePacketId => f.write_str("failed to encode packet ID"),
			Self::Nbt(e) => Display::fmt(&e, f),
			Self::Ident(e) => Display::fmt(&e, f),
			Self::Utf8(e) => Display::fmt(&e, f),
			Self::Other(o) => {
				f.write_str("unknown error occurred: ")?;
				f.write_str(&o)
			}
		}
	}
}

impl StdError for ProtocolError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::VarIntDecode(e) => Some(e),
			Self::Nbt(e) => Some(e),
			Self::Ident(e) => Some(e),
			Self::Utf8(e) => Some(e),
			_ => None,
		}
	}
}

impl From<IdentError> for ProtocolError {
	fn from(value: IdentError) -> Self {
		Self::Ident(value)
	}
}

impl From<IoError> for ProtocolError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}

impl From<NbtError> for ProtocolError {
	fn from(value: NbtError) -> Self {
		Self::Nbt(value)
	}
}

impl From<Utf8Error> for ProtocolError {
	fn from(value: Utf8Error) -> Self {
		Self::Utf8(value)
	}
}

impl From<VarIntDecodeError> for ProtocolError {
	fn from(value: VarIntDecodeError) -> Self {
		Self::VarIntDecode(value)
	}
}

pub trait Encode {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError>;

	fn encode_slice(slice: &[Self], mut w: impl Write) -> Result<(), ProtocolError>
	where
		Self: Sized,
	{
		for value in slice {
			value.encode(&mut w)?;
		}

		Ok(())
	}
}

pub trait Decode<'a>: Sized {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError>;
}

pub trait Packet: Debug {
	const ID: i32;
	const NAME: &'static str;
	const SIDE: PacketSide;
	const STATE: PacketState;

	fn encode_with_id(&self, mut w: impl Write) -> Result<(), ProtocolError>
	where
		Self: Encode,
	{
		VarInt(Self::ID)
			.encode(&mut w)
			.map_err(|_| ProtocolError::FailedToEncodePacketId)?;

		self.encode(w)
	}
}
