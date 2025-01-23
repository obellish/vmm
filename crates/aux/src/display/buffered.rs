use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	str::{Utf8Error, from_utf8},
};

use crate::{
	error::ComponentCreationError,
	vmm::board::Bus,
	vmm_tools::{
		bytes::words_to_bytes,
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, DisplayType},
	},
};

pub struct BufferedDisplay {
	buffer: Vec<u32>,
	words: u32,
	handler: Box<dyn FnMut(DecodedStr<'_>)>,
	hw_id: u64,
}

impl BufferedDisplay {
	pub fn new(
		capacity: u32,
		handler: impl FnMut(DecodedStr<'_>) + 'static,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let _: usize = capacity.try_into()?;

		if matches!(capacity, 0) {
			return Err(ComponentCreationError::CapacityCannotBeZero);
		}

		if !matches!(capacity % 4, 0) {
			return Err(ComponentCreationError::CapacityMustBeAligned);
		}

		let capacity = capacity / 4;

		Ok(Self {
			buffer: vec![0; (capacity - 1) as usize],
			words: capacity - 1,
			handler: Box::new(handler),
			hw_id,
		})
	}

	pub fn print_lossy(capacity: u32, hw_id: u64) -> Result<Self, ComponentCreationError> {
		Self::new(
			capacity,
			|message| match message {
				Ok(message) => print!("{message}"),
				Err(e) => print!("{}", String::from_utf8_lossy(e.bytes())),
			},
			hw_id,
		)
	}
}

impl Bus for BufferedDisplay {
	fn name(&self) -> &'static str {
		"Buffered Display"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			self.words * 4 + 4,
			DeviceCategory::Display(DisplayType::Buffered),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, _: u32, ex: &mut u16) -> u32 {
		*ex = AuxHwException::MemoryNotReadable.encode();
		0
	}

	fn write(&mut self, addr: u32, word: u32, ex: &mut u16) {
		let addr = addr / 4;

		if addr < self.words {
			self.buffer[addr as usize] = word;
			return;
		}

		match word {
			0xAA => {
				let bytes = words_to_bytes(&self.buffer);
				(self.handler)(
					from_utf8(bytes.as_slice()).map_err(|source| BufferedUtf8Error {
						source,
						bytes: &bytes,
					}),
				);
			}
			0xBB => {
				let bytes = words_to_bytes(&self.buffer);
				(self.handler)(Ok(&String::from_utf8_lossy(&bytes)));
			}
			0xFF => self.reset(),
			code => *ex = AuxHwException::UnknownOperation(code as u8).encode(),
		}
	}

	fn reset(&mut self) {
		self.buffer = vec![0; self.buffer.len()];
	}
}

impl Debug for BufferedDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("BufferedDisplay")
			.field("buffer", &self.buffer)
			.field("words", &self.words)
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}

#[derive(Debug)]
pub struct BufferedUtf8Error<'a> {
	source: Utf8Error,
	bytes: &'a [u8],
}

impl<'a> BufferedUtf8Error<'a> {
	#[must_use]
	pub const fn source(&self) -> &Utf8Error {
		&self.source
	}

	#[must_use]
	pub const fn bytes(&self) -> &'a [u8] {
		self.bytes
	}
}

impl Display for BufferedUtf8Error<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.source, f)
	}
}

impl StdError for BufferedUtf8Error<'_> {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		Some(&self.source)
	}
}

pub type DecodedStr<'a> = Result<&'a str, BufferedUtf8Error<'a>>;
