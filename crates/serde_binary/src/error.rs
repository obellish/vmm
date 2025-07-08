#[cfg(feature = "alloc")]
use alloc::{
	collections::TryReserveError,
	string::{String, ToString as _},
};
use core::{
	error::Error as CoreError,
	fmt::{Debug, Display, Error as FmtError, Formatter, Result as FmtResult},
	str::Utf8Error,
};
#[cfg(feature = "std")]
use std::io::Error as IoError;

use serde::{de::Error as DeError, ser::Error as SerError};

use super::Type;

#[derive(Debug)]
pub enum Error {
	UnexpectedEnd,
	ExcessData,
	BufferTooSmall,
	Allocation,
	Overflow,
	LimitReached,
	InvalidType(u8),
	VarIntTooLarge,
	WrongType(Type, &'static [Type]),
	NotOneChar,
	Format(FmtError),
	NotValidUtf8(Utf8Error),
	#[cfg(feature = "std")]
	Io(IoError),
	Custom,
	#[cfg(feature = "alloc")]
	Message(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::UnexpectedEnd => f.write_str("unexpected end of input"),
			Self::ExcessData => f.write_str("unexpected excess data"),
			Self::BufferTooSmall => f.write_str("output or scratch buffer was too small"),
			Self::Allocation => f.write_str("allocation failed"),
			Self::Overflow => {
				f.write_str("tried using more than ")?;
				Display::fmt(&usize::MAX, f)?;
				f.write_str(" bytes")
			}
			Self::LimitReached => f.write_str("configured size limit reached"),
			Self::InvalidType(v) => {
				write!(f, "invalid data type designator encountered: {v:#02X}")
			}
			Self::VarIntTooLarge => f.write_str("varint too large for the expected type"),
			Self::WrongType(found, expected) => {
				f.write_str("wrong type encountered, found ")?;
				Debug::fmt(&found, f)?;
				f.write_str(", but expected one of ")?;
				Debug::fmt(&expected, f)
			}
			Self::NotOneChar => f.write_str("string not exactly one character"),
			Self::Format(..) => f.write_str("value formatting error"),
			Self::NotValidUtf8(..) => f.write_str("string was not valid utf-8"),
			#[cfg(feature = "std")]
			Self::Io(..) => f.write_str("io error"),
			Self::Custom => f.write_str("unknown error"),
			#[cfg(feature = "alloc")]
			Self::Message(message) => {
				f.write_str("custom error: ")?;
				f.write_str(message)
			}
		}
	}
}

impl CoreError for Error {
	fn source(&self) -> Option<&(dyn CoreError + 'static)> {
		match self {
			Self::Format(e) => Some(e),
			Self::NotValidUtf8(e) => Some(e),
			#[cfg(feature = "std")]
			Self::Io(e) => Some(e),
			_ => None,
		}
	}
}

impl DeError for Error {
	#[cfg(feature = "alloc")]
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::Message(msg.to_string())
	}

	#[cfg(not(feature = "alloc"))]
	fn custom<T>(_: T) -> Self
	where
		T: Display,
	{
		Self::Custom
	}
}

impl SerError for Error {
	#[cfg(feature = "alloc")]
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::Message(msg.to_string())
	}

	#[cfg(not(feature = "alloc"))]
	fn custom<T>(_: T) -> Self
	where
		T: Display,
	{
		Self::Custom
	}
}

impl From<FmtError> for Error {
	fn from(value: FmtError) -> Self {
		Self::Format(value)
	}
}

impl From<Utf8Error> for Error {
	fn from(value: Utf8Error) -> Self {
		Self::NotValidUtf8(value)
	}
}

#[cfg(feature = "std")]
impl From<IoError> for Error {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}

#[cfg(feature = "alloc")]
impl From<TryReserveError> for Error {
	fn from(_: TryReserveError) -> Self {
		Self::Allocation
	}
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
