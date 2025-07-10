#[cfg(test)]
mod tests;

use core::mem;
#[cfg(feature = "std")]
use std::io::{BufReader, prelude::*};

use super::{Buffer, Error, Result};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct IoWriter<W> {
	writer: W,
}

impl<W> IoWriter<W> {
	pub const fn new(writer: W) -> Self {
		Self { writer }
	}
}

#[cfg(feature = "std")]
impl<W: Write> Output for IoWriter<W> {
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		self.writer.write_all(&[byte])?;
		Ok(())
	}

	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		self.writer.write_all(bytes)?;
		Ok(())
	}
}

#[derive(Debug)]
#[cfg(feature = "std")]
pub struct IoReader<R> {
	reader: BufReader<R>,
	next_byte: Option<u8>,
}

#[cfg(feature = "std")]
impl<R: Read> IoReader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			reader: BufReader::new(reader),
			next_byte: None,
		}
	}
}

#[cfg(feature = "std")]
impl<'de, R: Read> Input<'de> for IoReader<R> {
	fn peek_byte(&mut self) -> Result<u8> {
		let byte = Input::read_byte(self)?;
		self.next_byte = Some(byte);
		Ok(byte)
	}

	fn read_byte(&mut self) -> Result<u8> {
		if let Some(byte) = self.next_byte.take() {
			Ok(byte)
		} else {
			let mut bytes = self.reader.by_ref().bytes();
			let byte = bytes.next().ok_or(Error::UnexpectedEnd)??;
			Ok(byte)
		}
	}

	fn read_exact(&mut self, mut buffer: &mut [u8]) -> Result<()> {
		if buffer.is_empty() {
			return Ok(());
		}

		if let Some(byte) = self.next_byte.take() {
			let (first, remaining) = buffer.split_first_mut().ok_or(Error::BufferTooSmall)?;
			*first = byte;
			buffer = remaining;
		}

		match self.reader.read_exact(buffer) {
			Err(e) if matches!(e.kind(), std::io::ErrorKind::UnexpectedEof) => {
				return Err(Error::UnexpectedEnd);
			}
			res => res?,
		}

		Ok(())
	}

	fn skip_bytes(&mut self, mut len: usize) -> Result<()> {
		if matches!(len, 0) {
			return Ok(());
		}

		if self.next_byte.take().is_some() {
			len -= 1;
		}

		let to_write = u64::try_from(len).map_err(|_| Error::Overflow)?;
		let mut skip = self.reader.by_ref().take(to_write);
		let result = std::io::copy(&mut skip, &mut std::io::sink());

		match result {
			Err(e) if matches!(e.kind(), std::io::ErrorKind::UnexpectedEof) => {
				return Err(Error::UnexpectedEnd);
			}
			Ok(bytes) if bytes != to_write => return Err(Error::UnexpectedEnd),
			res => res?,
		};

		Ok(())
	}

	fn read_bytes<B: Buffer>(
		&mut self,
		mut len: usize,
		buffer: Option<&mut B>,
	) -> Result<Option<&'de [u8]>> {
		if matches!(len, 0) {
			return Ok(Some(&[]));
		}

		let buffer = buffer.ok_or(Error::BufferTooSmall)?;
		if let Some(byte) = self.next_byte.take() {
			buffer.push(byte)?;
			len -= 1;
		}

		let write = buffer.reserve_slice(len)?;
		match self.reader.read_exact(write) {
			Err(e) if matches!(e.kind(), std::io::ErrorKind::UnexpectedEof) => {
				return Err(Error::UnexpectedEnd);
			}
			res => res?,
		}

		Ok(None)
	}
}

#[derive(Debug, Clone)]
pub struct SizeLimit<T> {
	inner: T,
	limit: usize,
}

impl<T> SizeLimit<T> {
	pub const fn new(inner: T, limit: usize) -> Self {
		Self { inner, limit }
	}

	pub fn into_inner(self) -> T {
		self.inner
	}
}

impl<'de, T> Input<'de> for SizeLimit<T>
where
	T: Input<'de>,
{
	fn peek_byte(&mut self) -> Result<u8> {
		if matches!(self.limit, 0) {
			Err(Error::LimitReached)
		} else {
			self.inner.peek_byte()
		}
	}

	fn read_byte(&mut self) -> Result<u8> {
		if matches!(self.limit, 0) {
			Err(Error::LimitReached)
		} else {
			self.limit -= 1;

			self.inner.read_byte()
		}
	}

	fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
		if self.limit < buffer.len() {
			Err(Error::LimitReached)
		} else {
			self.limit -= buffer.len();

			self.inner.read_exact(buffer)
		}
	}

	fn skip_bytes(&mut self, len: usize) -> Result<()> {
		if self.limit < len {
			Err(Error::LimitReached)
		} else {
			self.limit -= len;

			self.inner.skip_bytes(len)
		}
	}

	fn read_bytes<B: Buffer>(
		&mut self,
		len: usize,
		buffer: Option<&mut B>,
	) -> Result<Option<&'de [u8]>> {
		if self.limit < len {
			Err(Error::LimitReached)
		} else {
			self.limit -= len;

			self.inner.read_bytes(len, buffer)
		}
	}
}

impl<T: Output> Output for SizeLimit<T> {
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		if matches!(self.limit, 0) {
			Err(Error::LimitReached)
		} else {
			self.limit -= 1;

			self.inner.write_byte(byte)
		}
	}
}

pub trait Input<'de> {
	fn peek_byte(&mut self) -> Result<u8>;

	fn read_byte(&mut self) -> Result<u8>;

	fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()>;

	fn skip_bytes(&mut self, len: usize) -> Result<()>;

	fn read_bytes<B: Buffer>(
		&mut self,
		len: usize,
		buffer: Option<&mut B>,
	) -> Result<Option<&'de [u8]>>;
}

impl<'de> Input<'de> for &'de [u8] {
	fn peek_byte(&mut self) -> Result<u8> {
		self.first().copied().ok_or(Error::UnexpectedEnd)
	}

	fn read_byte(&mut self) -> Result<u8> {
		let (byte, remaining) = self.split_first().ok_or(Error::UnexpectedEnd)?;
		*self = remaining;
		Ok(*byte)
	}

	fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
		let (slice, remaining) = self
			.split_at_checked(buffer.len())
			.ok_or(Error::UnexpectedEnd)?;
		*self = remaining;
		buffer.copy_from_slice(slice);
		Ok(())
	}

	fn skip_bytes(&mut self, len: usize) -> Result<()> {
		let (_, remaining) = self.split_at_checked(len).ok_or(Error::UnexpectedEnd)?;
		*self = remaining;
		Ok(())
	}

	fn read_bytes<B: Buffer>(
		&mut self,
		len: usize,
		_: Option<&mut B>,
	) -> Result<Option<&'de [u8]>> {
		let (slice, remaining) = self.split_at_checked(len).ok_or(Error::UnexpectedEnd)?;
		*self = remaining;

		Ok(Some(slice))
	}
}

pub trait Output {
	fn write_byte(&mut self, byte: u8) -> Result<()>;

	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		bytes.iter().try_for_each(|b| self.write_byte(*b))
	}
}

impl Output for &mut [u8] {
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		if self.is_empty() {
			return Err(Error::BufferTooSmall);
		}

		let (write, remaining) = mem::take(self)
			.split_first_mut()
			.ok_or(Error::BufferTooSmall)?;
		*write = byte;
		*self = remaining;

		Ok(())
	}

	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		if self.is_empty() {
			return Err(Error::BufferTooSmall);
		}

		let (write, remaining) = mem::take(self)
			.split_at_mut_checked(bytes.len())
			.ok_or(Error::BufferTooSmall)?;

		write.copy_from_slice(bytes);
		*self = remaining;

		Ok(())
	}
}

#[cfg(feature = "alloc")]
impl Output for alloc::vec::Vec<u8> {
	fn write_byte(&mut self, byte: u8) -> Result<()> {
		self.push(byte);
		Ok(())
	}

	fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
		self.extend_from_slice(bytes);
		Ok(())
	}
}
