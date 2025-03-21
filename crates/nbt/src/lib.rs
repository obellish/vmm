#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod compound;
pub mod deserializer;
pub mod tag;

use std::{fmt::Display, io::Error as IoError};
use cesu8::Cesu8DecodingError;
use serde::{de, ser};
use thiserror::Error;

pub const END_ID: u8 = 0x00;
pub const BYTE_ID: u8 = 0x01;
pub const SHORT_ID: u8 = 0x02;
pub const INT_ID: u8 = 0x03;
pub const LONG_ID: u8 = 0x04;
pub const FLOAT_ID: u8 = 0x05;
pub const DOUBLE_ID: u8 = 0x06;
pub const BYTE_ARRAY_ID: u8 = 0x07;
pub const STRING_ID: u8 = 0x08;
pub const LIST_ID: u8 = 0x09;
pub const COMPOUND_ID: u8 = 0x0A;
pub const INT_ARRAY_ID: u8 = 0x0B;
pub const LONG_ARRAY_ID: u8 = 0x0C;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Nbt {
	pub name: String,
}

#[derive(Debug, Error)]
pub enum Error {
	#[error("The root tag of the NBT file is not a compound tag. Received tag id: {0}")]
	NoRootCompound(u8),
	#[error("Encountered an unknown NBT tag id: {0}.")]
	UnknownTagId(u8),
	#[error("Failed to Cesu 8 Decode: {0}")]
	Cesu8Decoding(#[from] Cesu8DecodingError),
	#[error("Serde error: {0}")]
	Serde(String),
	#[error("NBT doesn't support this type: {0}")]
	UnsupportedType(String),
	#[error("NBT reading was cut short: {0}")]
	Incomplete(#[from] IoError),
	#[error("Negative list length: {0}")]
	NegativeLength(i32),
	#[error("Length too large: {0}")]
	LargeLength(usize),
}

impl de::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Serde(msg.to_string())
	}
}

impl ser::Error for Error {
	fn custom<T: Display>(msg: T) -> Self {
		Self::Serde(msg.to_string())
	}
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
