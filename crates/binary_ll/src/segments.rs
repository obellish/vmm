use core::{cmp::min, marker::PhantomData};

use vmm_binary_io::Read;

use super::{Decoder, Error, Header};

#[derive(Default)]
#[repr(transparent)]
pub struct Bytes(());

impl Parser for Bytes {
	type Error = core::convert::Infallible;
	type Item = [u8];

	fn parse<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a Self::Item, Self::Error> {
		Ok(bytes)
	}
}

#[derive(Default)]
pub struct Text {
	stored: usize,
	buffer: [u8; 3],
}

impl Parser for Text {
	type Error = core::str::Utf8Error;
	type Item = str;

	fn parse<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a Self::Item, Self::Error> {
		if bytes.len() <= self.stored {
			return Ok("");
		}

		bytes[..self.stored].clone_from_slice(&self.buffer[..self.stored]);

		Ok(match core::str::from_utf8(bytes) {
			Ok(s) => {
				self.stored = 0;
				s
			}
			Err(e) => {
				let valid_len = e.valid_up_to();
				let invalid_len = bytes.len() - valid_len;

				if invalid_len > self.buffer.len() {
					return Err(e);
				}

				self.buffer[..invalid_len].clone_from_slice(&bytes[valid_len..]);
				self.stored = invalid_len;

				core::str::from_utf8(&bytes[..valid_len])?
			}
		})
	}

	fn saved(&self) -> usize {
		self.stored
	}
}

pub struct Segment<'r, R, P> {
	reader: &'r mut Decoder<R>,
	unread: usize,
	offset: usize,
	parser: P,
}

impl<R: Read, P: Parser> Segment<'_, R, P> {
	pub fn left(&self) -> usize {
		self.unread + self.parser.saved()
	}

	pub fn pull<'a>(
		&mut self,
		buffer: &'a mut [u8],
	) -> Result<Option<&'a P::Item>, Error<R::Error>> {
		let prev = self.parser.saved();
		match self.unread {
			0 if matches!(prev, 0) => return Ok(None),
			0 => return Err(Error::Syntax(self.offset)),
			_ => {}
		}

		let size = min(buffer.len(), prev + self.unread);
		let full = &mut buffer[..size];
		let next = &mut full[min(size, prev)..];

		self.reader.read_exact(next)?;
		self.unread -= next.len();

		self.parser
			.parse(full)
			.or(Err(Error::Syntax(self.offset)))
			.map(Some)
	}
}

pub struct Segments<'r, R, P: ?Sized> {
	reader: &'r mut Decoder<R>,
	state: State,
	parser: PhantomData<P>,
	unwrap: fn(Header) -> Option<Option<usize>>,
}

impl<'r, R, P: ?Sized> Segments<'r, R, P> {
	pub(crate) fn new(
		decoder: &'r mut Decoder<R>,
		unwrap: fn(Header) -> Option<Option<usize>>,
	) -> Self {
		Self {
			reader: decoder,
			state: State::Initial,
			parser: PhantomData,
			unwrap,
		}
	}
}

impl<R: Read, P: Parser> Segments<'_, R, P> {
	pub fn pull(&mut self) -> Result<Option<Segment<'_, R, P>>, Error<R::Error>> {
		while !matches!(self.state, State::Finished) {
			let offset = self.reader.offset();
			match self.reader.pull()? {
				Header::Break => {
					self.state = State::Finished;
					return Ok(None);
				}
				header => match (self.unwrap)(header) {
					None => return Err(Error::Syntax(offset)),
					Some(None) => {
						if matches!(self.state, State::Initial) {
							self.state = State::Continue;
						} else {
							return Err(Error::Syntax(offset));
						}
					}
					Some(Some(len)) => {
						if matches!(self.state, State::Initial) {
							self.state = State::Finished;
						}

						return Ok(Some(Segment {
							reader: self.reader,
							unread: len,
							offset,
							parser: P::default(),
						}));
					}
				},
			}
		}

		Ok(None)
	}
}

#[derive(PartialEq, Eq)]
enum State {
	Initial,
	Continue,
	Finished,
}

pub trait Parser: Default {
	type Item: ?Sized;

	type Error;

	fn parse<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a Self::Item, Self::Error>;

	fn saved(&self) -> usize {
		0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn t(data: &[u8], len: usize) {
		let mut dec = Decoder::from_reader(data);
		let mut segs = Segments::<_, Bytes>::new(&mut dec, |header| match header {
			Header::Bytes(len) => Some(len),
			_ => None,
		});

		while let Some(mut seg) = segs.pull().unwrap() {
			let mut b = [0; 1];
			assert_eq!(seg.pull(&mut b).unwrap(), Some(&b"0"[..]));
		}

		assert_eq!(len, dec.offset());
	}

	#[test]
	fn segments() {
		t(b"\x410\x00", 2);
		t(b"\x5f\x410\xff\x00", 4);
	}
}
