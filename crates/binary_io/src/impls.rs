#[cfg(not(feature = "std"))]
use core::{
	error::Error as CoreError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use super::{Read, Write};

#[cfg(feature = "std")]
impl<T: std::io::Read> Read for T {
	type Error = std::io::Error;

	fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
		self.read_exact(data)
	}
}

#[cfg(feature = "std")]
impl<T: std::io::Write> Write for T {
	type Error = std::io::Error;

	fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
		self.write_all(data)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.flush()
	}
}

#[cfg(not(feature = "std"))]
impl<T> Read for &mut T
where
	T: ?Sized + Read,
{
	type Error = T::Error;

	fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
		(**self).read_exact(data)
	}
}

#[cfg(not(feature = "std"))]
impl<T> Write for &mut T
where
	T: ?Sized + Write,
{
	type Error = T::Error;

	fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
		(**self).write_all(data)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		(**self).flush()
	}
}

#[cfg(not(feature = "std"))]
impl Read for &[u8] {
	type Error = EndOfFileError;

	fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
		if data.len() > self.len() {
			return Err(EndOfFileError(()));
		}

		let (prefix, suffix) = self.split_at(data.len());
		data.copy_from_slice(prefix);
		*self = suffix;
		Ok(())
	}
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
impl Write for alloc::vec::Vec<u8> {
	type Error = core::convert::Infallible;

	fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
		self.extend_from_slice(data);
		Ok(())
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		Ok(())
	}
}

#[cfg(not(feature = "std"))]
impl Write for &mut [u8] {
	type Error = OutOfSpaceError;

	fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
		if data.len() > self.len() {
			return Err(OutOfSpaceError(()));
		}

		let (prefix, suffix) = core::mem::take(self).split_at_mut(data.len());
		prefix.copy_from_slice(data);
		*self = suffix;
		Ok(())
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		Ok(())
	}
}

#[cfg(not(feature = "std"))]
#[derive(Debug, Clone)]
pub struct EndOfFileError(());

#[cfg(not(feature = "std"))]
impl Display for EndOfFileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("end of file")
	}
}

#[cfg(not(feature = "std"))]
impl CoreError for EndOfFileError {}

#[derive(Debug, Clone)]
#[cfg(not(feature = "std"))]
pub struct OutOfSpaceError(());

#[cfg(not(feature = "std"))]
impl Display for OutOfSpaceError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("out of space")
	}
}

#[cfg(not(feature = "std"))]
impl CoreError for OutOfSpaceError {}
