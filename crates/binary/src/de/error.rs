use alloc::string::{String, ToString as _};
use core::{
	error::Error as CoreError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use serde::de::Error as DeError;

#[derive(Debug, Clone)]
pub enum Error<T> {
	Io(T),
	Syntax(usize),
	Semantic(Option<usize>, String),
	RecursionLimitExceeded,
}

impl<T> Error<T> {
	pub fn semantic(offset: impl Into<Option<usize>>, message: impl Into<String>) -> Self {
		Self::Semantic(offset.into(), message.into())
	}
}

impl<T: Debug> Display for Error<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self, f)
	}
}

impl<T: Debug> CoreError for Error<T> {}

impl<T: Debug> DeError for Error<T> {
	fn custom<U>(msg: U) -> Self
	where
		U: Display,
	{
		Self::Semantic(None, msg.to_string())
	}
}

impl<T> From<vmm_binary_ll::Error<T>> for Error<T> {
	fn from(value: vmm_binary_ll::Error<T>) -> Self {
		match value {
			vmm_binary_ll::Error::Io(x) => Self::Io(x),
			vmm_binary_ll::Error::Syntax(x) => Self::Syntax(x),
		}
	}
}

impl<T> From<T> for Error<T> {
	fn from(value: T) -> Self {
		Self::Io(value)
	}
}
