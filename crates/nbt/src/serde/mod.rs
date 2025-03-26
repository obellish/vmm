mod de;
mod ser;
#[cfg(test)]
mod tests;

use std::fmt::Display;

pub use self::ser::*;
use super::Error;

impl serde::de::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::owned(format!("{msg}"))
	}
}

impl serde::ser::Error for Error {
	fn custom<T>(msg: T) -> Self
	where
		T: Display,
	{
		Self::owned(format!("{msg}"))
	}
}
