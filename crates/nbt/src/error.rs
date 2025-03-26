use std::{
	borrow::Cow,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IoError,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Error {
	cause: Box<Cause>,
}

impl Error {
	pub(crate) fn owned(message: impl Into<String>) -> Self {
		Self {
			cause: Box::new(Cause::Other(Cow::Owned(message.into()))),
		}
	}

	pub(crate) fn r#static(message: &'static str) -> Self {
		Self {
			cause: Box::new(Cause::Other(Cow::Borrowed(message))),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match &*self.cause {
			Cause::Io(e) => Display::fmt(&e, f),
			Cause::Other(s) => f.write_str(s),
		}
	}
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		let Cause::Io(e) = &*self.cause else {
			return None;
		};

		Some(e)
	}
}

impl From<IoError> for Error {
	fn from(value: IoError) -> Self {
		Self {
			cause: Box::new(Cause::Io(value)),
		}
	}
}

#[derive(Debug)]
enum Cause {
	Io(IoError),
	Other(Cow<'static, str>),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
