use core::result;
#[cfg(feature = "std")]
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error(i32);

impl Error {
	#[must_use]
	pub const fn new(errno: i32) -> Self {
		Self(errno)
	}

	#[must_use]
	#[cfg(feature = "std")]
	pub fn last() -> Self {
		Self::from(std::io::Error::last_os_error())
	}

	#[must_use]
	pub const fn errno(self) -> i32 {
		self.0
	}
}

#[cfg(feature = "std")]
impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		std::io::Error::from_raw_os_error(self.errno()).fmt(f)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::new(value.raw_os_error().unwrap_or_default())
	}
}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
	fn from(value: Error) -> Self {
		Self::from_raw_os_error(value.errno())
	}
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[cfg(feature = "std")]
pub fn errno_result<T>() -> Result<T> {
	Err(Error::last())
}

#[cfg(all(test, feature = "std"))]
mod tests {}
