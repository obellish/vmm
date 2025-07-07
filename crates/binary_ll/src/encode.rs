use core::cmp;

use vmm_binary_io::Write;

use super::{Header, Major, Minor, Title};

#[repr(transparent)]
pub struct Encoder<W>(W);

impl<W> Encoder<W> {
	pub const fn from_writer(writer: W) -> Self {
		Self(writer)
	}

	pub fn into_inner(self) -> W {
		self.0
	}
}

impl<W: Write> Encoder<W> {
	pub fn push(&mut self, header: Header) -> Result<(), W::Error> {
		let title = Title::from(header);

		let major = match title.0 {
			Major::Positive => 0,
			Major::Negative => 1,
			Major::Bytes => 2,
			Major::Text => 3,
			Major::Array => 4,
			Major::Map => 5,
			Major::Tag => 6,
			Major::Other => 7,
		};

		let minor = match title.1 {
			Minor::This(x) => x,
			Minor::Next1(..) => 24,
			Minor::Next2(..) => 25,
			Minor::Next4(..) => 26,
			Minor::Next8(..) => 27,
			Minor::More => 31,
		};

		self.0.write_all(&[major << 5 | minor])?;
		self.0.write_all(title.1.as_ref())
	}

	pub fn bytes(
		&mut self,
		value: &[u8],
		segment: impl Into<Option<usize>>,
	) -> Result<(), W::Error> {
		let max: usize = segment.into().unwrap_or(value.len());
		let max = cmp::max(max, 1);

		if max >= value.len() {
			self.push(Header::Bytes(Some(value.len())))?;
			self.write_all(value)?;
		} else {
			self.push(Header::Bytes(None))?;

			for chunk in value.chunks(max) {
				self.push(Header::Bytes(Some(chunk.len())))?;
				self.write_all(chunk)?;
			}

			self.push(Header::Break)?;
		}

		Ok(())
	}

	pub fn text(&mut self, value: &str, segment: impl Into<Option<usize>>) -> Result<(), W::Error> {
		let max: usize = segment.into().unwrap_or(value.len());
		let max = cmp::max(max, 4);

		if max >= value.len() {
			self.push(Header::Text(Some(value.len())))?;
			self.write_all(value.as_bytes())?;
		} else {
			self.push(Header::Text(None))?;

			let mut bytes = value.as_bytes();
			while !bytes.is_empty() {
				let mut len = cmp::min(bytes.len(), max);
				while len > 0 && core::str::from_utf8(&bytes[..len]).is_err() {
					len -= 1;
				}

				let (prefix, suffix) = bytes.split_at(len);
				self.push(Header::Text(Some(prefix.len())))?;
				self.write_all(prefix)?;
				bytes = suffix;
			}

			self.push(Header::Break)?;
		}

		Ok(())
	}
}

impl<W> From<W> for Encoder<W> {
	fn from(value: W) -> Self {
		Self::from_writer(value)
	}
}

impl<W: Write> Write for Encoder<W> {
	type Error = W::Error;

	fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error> {
		self.0.write_all(data)
	}

	fn flush(&mut self) -> Result<(), Self::Error> {
		self.0.flush()
	}
}
