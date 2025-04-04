use std::{
	io::Write,
	mem,
	ops::{Deref, DerefMut},
};

use super::{Bounded, Decode, Encode, ProtocolError};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RawBytes<'a>(pub &'a [u8]);

impl<'a> Decode<'a> for RawBytes<'a> {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		Ok(Self(mem::take(r)))
	}
}

impl<'a, const MAX_BYTES: usize> Decode<'a> for Bounded<RawBytes<'a>, MAX_BYTES> {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		assert!(
			r.len() <= MAX_BYTES,
			"remainder of input exceeds max of {MAX_BYTES} bytes (got {} bytes)",
			r.len()
		);

		Ok(Self(RawBytes::decode(r)?))
	}
}

impl<'a> Deref for RawBytes<'a> {
	type Target = &'a [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'a> DerefMut for RawBytes<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Encode for RawBytes<'_> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		Ok(w.write_all(self.0)?)
	}
}

impl<const MAX_BYTES: usize> Encode for Bounded<RawBytes<'_>, MAX_BYTES> {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		assert!(
			self.len() <= MAX_BYTES,
			"cannot encode more than {MAX_BYTES} raw bytes (got {} bytes)",
			self.len()
		);

		self.0.encode(w)
	}
}

impl<'a> From<&'a [u8]> for RawBytes<'a> {
	fn from(value: &'a [u8]) -> Self {
		Self(value)
	}
}

impl<'a> From<RawBytes<'a>> for &'a [u8] {
	fn from(value: RawBytes<'a>) -> Self {
		value.0
	}
}
