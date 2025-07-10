use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
};

use serde::{de::Error as DeError, ser::Error as SerError};

use super::ContainerFormat;

#[derive(Debug)]
pub enum Error {
	Custom(String),
	NotSupported(&'static str),
	Deserialization(&'static str),
	UnexpectedDeserializationFormat(&'static str, ContainerFormat, &'static str),
	Incompatible(String, String),
	UnknownFormat,
	UnknownFormatInContainer(String),
	MissingVariants(Vec<String>),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Custom(s) => f.write_str(s),
			Self::NotSupported(s) => {
				f.write_str("not supported: ")?;
				f.write_str(s)
			}
			Self::Deserialization(s) => {
				f.write_str("failed to deserialize ")?;
				f.write_str(s)
			}
			Self::UnexpectedDeserializationFormat(s, format, into) => {
				f.write_str("in container ")?;
				f.write_str(s)?;
				f.write_str(", recorded value for serialization format ")?;
				Debug::fmt(&format, f)?;
				f.write_str(" failed to deserialize into ")?;
				f.write_str(into)
			}
			Self::Incompatible(first, second) => {
				f.write_str("incompatible formats detected: ")?;
				f.write_str(first)?;
				f.write_char(' ')?;
				f.write_str(second)
			}
			Self::UnknownFormat => f.write_str("incomplete tracing detected"),
			Self::UnknownFormatInContainer(container) => {
				f.write_str("incomplete tracing detected inside container: ")?;
				f.write_str(container)
			}
			Self::MissingVariants(variants) => {
				f.write_str("missing variants detected for specific enums: ")?;
				Debug::fmt(&variants, f)
			}
		}
	}
}

impl StdError for Error {}

impl SerError for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::Custom(msg.to_string())
	}
}

impl DeError for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::Custom(msg.to_string())
	}
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
