use std::{io::Write, str::FromStr};

use vmm_text::Text;

use crate::{Bounded, Decode, Encode, ProtocolError, VarInt};

const DEFAULT_MAX_STRING_CHARS: usize = 32767;
const MAX_TEXT_CHARS: usize = 262144;

impl<'a> Decode<'a> for &'a str {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		Ok(Bounded::<_, DEFAULT_MAX_STRING_CHARS>::decode(r)?.0)
	}
}

impl Encode for str {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		Bounded::<_, DEFAULT_MAX_STRING_CHARS>(self).encode(w)
	}
}

impl<'a, const MAX_CHARS: usize> Decode<'a> for Bounded<&'a str, MAX_CHARS> {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(len >= 0, "attempt to decode string with negative length");
		let len = len as usize;
		debug_assert!(
			len <= r.len(),
			"not enough data remaining ({} bytes) to decode string of {len} bytes",
			r.len()
		);

		let (res, remaining) = r.split_at(len);
		let res = std::str::from_utf8(res)?;

		let char_count = res.encode_utf16().count();
		debug_assert!(
			char_count <= MAX_CHARS,
			"char count of string exceeds maximum (expected <= {MAX_CHARS}, got {char_count})"
		);

		*r = remaining;

		Ok(Self(res))
	}
}

impl<const MAX_CHARS: usize> Encode for Bounded<&'_ str, MAX_CHARS> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let char_count = self.encode_utf16().count();

		debug_assert!(
			char_count <= MAX_CHARS,
			"char count of string exceeds maximum (expected <= {MAX_CHARS}, got {char_count})"
		);

		VarInt(self.len() as i32).encode(&mut w)?;
		Ok(w.write_all(self.as_bytes())?)
	}
}

impl Decode<'_> for String {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		<&str>::decode(r).map(ToOwned::to_owned)
	}
}

impl Encode for String {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_str().encode(w)
	}
}

impl<const MAX_CHARS: usize> Decode<'_> for Bounded<String, MAX_CHARS> {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		Ok(Bounded(Bounded::<&str, MAX_CHARS>::decode(r)?.0.to_owned()))
	}
}

impl<const MAX_CHARS: usize> Encode for Bounded<String, MAX_CHARS> {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		Bounded::<_, MAX_CHARS>(self.as_str()).encode(w)
	}
}

impl Decode<'_> for Box<str> {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		<&str>::decode(r).map(Into::into)
	}
}

impl<const MAX_CHARS: usize> Decode<'_> for Bounded<Box<str>, MAX_CHARS> {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		Ok(Bounded(Bounded::<&str, MAX_CHARS>::decode(r)?.0.into()))
	}
}

impl Decode<'_> for Text {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		let str = Bounded::<&str, MAX_TEXT_CHARS>::decode(r)?.0;

		Ok(Self::from_str(str).map_err(|e| ProtocolError::Other(e.to_string()))?)
	}
}

impl Encode for Text {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		let s = serde_json::to_string(self).map_err(|e| ProtocolError::Other(e.to_string()))?;

		Bounded::<_, MAX_TEXT_CHARS>(s).encode(w)
	}
}
