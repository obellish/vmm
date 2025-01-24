mod kv_map;
pub mod collections {
	pub use super::kv_map::*;
}

use alloc::string::String;
use core::fmt::{Error as FmtError, Write};

use thiserror::Error;
pub use winter_utils::{
	ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, SliceReader,
	uninit_vector,
};

use super::Word;

#[derive(Debug, Error)]
pub enum HexParseError {
	#[error("expected hex data to have length {expected}, including the 0x prefix, found {actual}")]
	InvalidLength { expected: usize, actual: usize },
	#[error("hex encoded data must start with the 0x prefix")]
	MissingPrefix,
	#[error("hex encoded data must only contain characters [a-zA-Z0-9]")]
	InvalidChar,
	#[error("hex encoded values of a Digest must be inside the field modulus")]
	OutOfRange,
}

pub fn hex_to_bytes<const N: usize>(value: &str) -> Result<[u8; N], HexParseError> {
	let expected: usize = (N * 2) + 2;
	if value.len() != expected {
		return Err(HexParseError::InvalidLength {
			expected,
			actual: value.len(),
		});
	}

	if !value.starts_with("0x") {
		return Err(HexParseError::MissingPrefix);
	}

	let mut data = value.bytes().skip(2).map(|v| match v {
		b'0'..=b'9' => Ok(v - b'0'),
		b'a'..=b'f' => Ok(v - b'a' + 10),
		b'A'..=b'F' => Ok(v - b'A' + 10),
		_ => Err(HexParseError::InvalidChar),
	});

	let mut decoded = [0; N];
	for byte in &mut decoded {
		let high = data.next().unwrap()?;
		let low = data.next().unwrap()?;
		*byte = (high << 4) + low;
	}

	Ok(decoded)
}

pub fn word_to_hex(w: Word) -> Result<String, FmtError> {
	let mut s = String::new();

	for byte in w.iter().flat_map(winter_utils::Serializable::to_bytes) {
		write!(s, "{byte:02x}")?;
	}

	Ok(s)
}

#[must_use]
pub fn bytes_to_hex_string<const N: usize>(data: [u8; N]) -> String {
	let mut s = String::with_capacity(N + 2);

	s.push_str("0x");
	for byte in data {
		write!(s, "{byte:02x}").expect("formatting hex failed");
	}

	s
}
