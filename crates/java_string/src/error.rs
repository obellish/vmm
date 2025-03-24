use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utf8Error {
	pub(crate) valid_up_to: usize,
	pub(crate) error_len: Option<u8>,
}

#[expect(clippy::trivially_copy_pass_by_ref)]
impl Utf8Error {
	#[must_use]
	pub const fn valid_up_to(&self) -> usize {
		self.valid_up_to
	}

	#[must_use]
	pub const fn error_len(&self) -> Option<usize> {
		match self.error_len {
			Some(len) => Some(len as usize),
			None => None,
		}
	}

	pub(crate) const fn from_std(value: std::str::Utf8Error) -> Self {
		Self {
			valid_up_to: value.valid_up_to(),
			error_len: match value.error_len() {
				Some(error_len) => Some(error_len as u8),
				None => None,
			},
		}
	}
}

impl Display for Utf8Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid utf-8 ")?;

		if let Some(error_len) = self.error_len {
			f.write_str("sequence of ")?;
			Display::fmt(&error_len, f)?;
			f.write_str(" bytes from index ")?;
		} else {
			f.write_str("byte sequence from index ")?;
		}
		Display::fmt(&self.valid_up_to, f)
	}
}

impl Error for Utf8Error {}

impl From<std::str::Utf8Error> for Utf8Error {
	fn from(value: std::str::Utf8Error) -> Self {
		Self::from_std(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromUtf8Error {
	pub(crate) bytes: Vec<u8>,
	pub(crate) error: Utf8Error,
}

impl FromUtf8Error {
	#[allow(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	#[must_use]
	pub fn into_bytes(self) -> Vec<u8> {
		self.bytes
	}

	#[must_use]
	pub const fn utf8_error(&self) -> Utf8Error {
		self.error
	}
}

impl Display for FromUtf8Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.error, f)
	}
}

impl Error for FromUtf8Error {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.error)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError<E> {
	InvalidUtf8(Utf8Error),
	Other(E),
}

impl<E: Display> Display for ParseError<E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::InvalidUtf8(e) => Display::fmt(&e, f),
			Self::Other(e) => Display::fmt(&e, f),
		}
	}
}

impl<E> Error for ParseError<E>
where
	E: Error + 'static,
{
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::InvalidUtf8(e) => Some(e),
			Self::Other(e) => Some(e),
		}
	}
}

impl<E> From<Utf8Error> for ParseError<E> {
	fn from(value: Utf8Error) -> Self {
		Self::InvalidUtf8(value)
	}
}
