use vmm_binary_io::Read;

use super::{
	Header, Major, Minor, Segments, Title,
	segments::{Bytes, Text},
};

pub struct Decoder<R> {
	reader: R,
	offset: usize,
	buffer: Option<Title>,
}

impl<R> Decoder<R> {
	pub const fn from_reader(reader: R) -> Self {
		Self {
			reader,
			offset: 0,
			buffer: None,
		}
	}
}

impl<R: Read> Decoder<R> {
	fn pull_title(&mut self) -> Result<Title, Error<R::Error>> {
		if let Some(title) = self.buffer.take() {
			self.offset += title.1.as_ref().len() + 1;
			return Ok(title);
		}

		let mut prefix = [0u8; 1];
		self.read_exact(&mut prefix[..])?;

		let major = match prefix[0] >> 5 {
			0 => Major::Positive,
			1 => Major::Negative,
			2 => Major::Bytes,
			3 => Major::Text,
			4 => Major::Array,
			5 => Major::Map,
			6 => Major::Tag,
			7 => Major::Other,
			_ => unreachable!(),
		};

		let mut minor = match prefix[0] & 0b0001_1111 {
			x @ 0..24 => Minor::This(x),
			24 => Minor::Next1([0]),
			25 => Minor::Next2([0; 2]),
			26 => Minor::Next4([0; 4]),
			27 => Minor::Next8([0; 8]),
			31 => Minor::More,
			_ => return Err(Error::Syntax(self.offset - 1)),
		};

		self.read_exact(minor.as_mut())?;

		Ok(Title(major, minor))
	}

	fn push_title(&mut self, item: Title) {
		assert!(self.buffer.is_none());
		self.buffer = Some(item);
		self.offset -= item.1.as_ref().len() + 1;
	}

	pub fn pull(&mut self) -> Result<Header, Error<R::Error>> {
		let offset = self.offset;
		self.pull_title()?
			.try_into()
			.map_err(|_| Error::Syntax(offset))
	}

	pub fn push(&mut self, item: Header) {
		self.push_title(Title::from(item));
	}

	pub const fn offset(&self) -> usize {
		self.offset
	}

	pub fn bytes(&mut self, len: Option<usize>) -> Segments<'_, R, Bytes> {
		self.push(Header::Bytes(len));
		Segments::new(self, |header| match header {
			Header::Bytes(len) => Some(len),
			_ => None,
		})
	}

	pub fn text(&mut self, len: Option<usize>) -> Segments<'_, R, Text> {
		self.push(Header::Text(len));
		Segments::new(self, |header| match header {
			Header::Text(len) => Some(len),
			_ => None,
		})
	}
}

impl<R> From<R> for Decoder<R> {
	fn from(value: R) -> Self {
		Self::from_reader(value)
	}
}

impl<R: Read> Read for Decoder<R> {
	type Error = R::Error;

	fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
		assert!(self.buffer.is_none());
		self.reader.read_exact(data)?;
		self.offset += data.len();
		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Error<T> {
	Io(T),
	Syntax(usize),
}

impl<T> From<T> for Error<T> {
	fn from(value: T) -> Self {
		Self::Io(value)
	}
}
