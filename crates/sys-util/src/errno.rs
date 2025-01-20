use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	io,
};

/// Wrapper over [`errno`](http://man7.org/linux/man-pages/man3/errno.3.html)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(i32);

impl Error {
	#[must_use]
	pub const fn new(errno: i32) -> Self {
		Self(errno)
	}

	#[must_use]
	pub fn last() -> Self {
		io::Error::last_os_error().into()
	}

	#[must_use]
	pub const fn errno(self) -> i32 {
		self.0
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		io::Error::from_raw_os_error(self.errno()).fmt(f)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::new(value.raw_os_error().unwrap_or_default())
	}
}

impl From<Error> for io::Error {
	fn from(value: Error) -> Self {
		Self::from_raw_os_error(value.errno())
	}
}

pub fn errno_result<T>() -> Result<T> {
	Err(Error::last())
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
