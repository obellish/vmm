use alloc::string::{String, ToString as _};
use core::{
	error::Error as CoreError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use serde::ser::Error as SerError;

#[derive(Debug, Clone)]
pub enum Error<T> {
	Io(T),
	Value(String),
}

impl<T: Debug> Display for Error<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self, f)
	}
}

impl<T: Debug> CoreError for Error<T> {}

impl<T: Debug> SerError for Error<T> {
	fn custom<U>(msg: U) -> Self
	where
		U: Display,
	{
		Self::Value(msg.to_string())
	}
}

impl<T> From<T> for Error<T> {
	fn from(value: T) -> Self {
		Self::Io(value)
	}
}
