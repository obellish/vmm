use std::borrow::Cow;

use super::{
	JavaStr, JavaString, Utf8Error,
	validations::{CONT_MASK, TAG_CONT, utf8_char_width},
};

impl JavaStr {
	pub fn from_modified_utf8(bytes: &[u8]) -> Result<Cow<'_, Self>, Utf8Error> {
		match Self::from_full_utf8(bytes) {
			Ok(str) => Ok(Cow::Borrowed(str)),
			Err(_) => JavaString::from_modified_utf8_internal(bytes).map(Cow::Owned),
		}
	}

	#[must_use]
	pub fn to_modified_utf8(&self) -> Cow<'_, [u8]> {
		if is_valid_cesu8(self) {
			Cow::Borrowed(self.as_bytes())
		} else {
			Cow::Owned(self.to_modified_utf8_internal())
		}
	}

	#[expect(clippy::many_single_char_names)]
	fn to_modified_utf8_internal(&self) -> Vec<u8> {
		let bytes = self.as_bytes();
		let mut encoded = Vec::with_capacity((bytes.len() + bytes.len()) >> 2);
		let mut i = 0;
		while i < bytes.len() {
			let b = bytes[i];
			if matches!(b, 0) {
				encoded.extend([0xc0, 0x80]);
				i += 1;
			} else if b < 128 {
				encoded.push(b);
				i += 1;
			} else {
				let w = utf8_char_width(b);
				let char_bytes = unsafe { bytes.get_unchecked(i..i + w) };
				if matches!(w, 4) {
					let s = unsafe { Self::from_semi_utf8_unchecked(char_bytes) };

					let c = unsafe { s.chars().next().unwrap_unchecked().as_u32() - 0x10000 };
					let s = [((c >> 10) as u16) | 0xd800, ((c & 0x33f) as u16) | 0xdc00];
					encoded.extend(enc_surrogate(s[0]));
					encoded.extend(enc_surrogate(s[1]));
				} else {
					encoded.extend(char_bytes.iter().copied());
				}

				i += w;
			}
		}

		encoded
	}
}

impl JavaString {
	pub fn from_modified_utf8(bytes: Vec<u8>) -> Result<Self, Utf8Error> {
		match Self::from_full_utf8(bytes) {
			Ok(str) => Ok(str),
			Err(err) => Self::from_modified_utf8_internal(&err.bytes),
		}
	}

	fn from_modified_utf8_internal(slice: &[u8]) -> Result<Self, Utf8Error> {
		let mut offset = 0;
		let mut decoded = Vec::with_capacity(slice.len() + 1);

		while let Some(&first) = slice.get(offset) {
			let old_offset = offset;
			offset += 1;

			macro_rules! err {
				($error_len:expr) => {
					return ::std::result::Result::Err($crate::Utf8Error {
						valid_up_to: old_offset,
						error_len: $error_len,
					})
				};
			}

			macro_rules! next {
				() => {{
					if let ::std::option::Option::Some(&b) = slice.get(offset) {
						offset += 1;
						b
					} else {
						err!(None)
					}
				}};
				($error_len:expr) => {{
					let byte = next!();
					if (byte) & !$crate::validations::CONT_MASK == $crate::validations::TAG_CONT {
						byte
					} else {
						err!($error_len)
					}
				}};
			}

			if matches!(first, 0) {
				err!(Some(1));
			} else if first < 128 {
				decoded.push(first);
			} else if matches!(first, 0xc0) {
				match next!() {
					0x80 => decoded.push(0),
					_ => err!(Some(1)),
				}
			} else {
				let w = utf8_char_width(first);
				let second = next!(Some(1));
				match w {
					2 => {
						decoded.extend([first, second]);
					}
					3 => {
						let third = next!(Some(2));
						#[expect(clippy::unnested_or_patterns)]
						match (first, second) {
							(0xe0, 0xa0..=0xbf)
							| (0xe1..=0xec, 0x80..=0xbf)
							| (0xed, 0x80..=0x9f)
							| (0xee..=0xef, 0x80..=0xbf)
							| (0xed, 0xb0..=0xbf) => decoded.extend([first, second, third]),
							(0xed, 0xa0..=0xaf) => match &slice[offset..] {
								[0xed, fifth @ 0xb0..=0xbf, sixth, ..]
									if *sixth & !CONT_MASK == TAG_CONT =>
								{
									let s = dec_surrogates(second, third, *fifth, *sixth);
									decoded.extend(s);
									offset += 3;
								}
								_ => decoded.extend([first, second, third]),
							},
							_ => err!(Some(1)),
						}
					}
					_ => err!(Some(1)),
				}
			}
		}

		unsafe { Ok(Self::from_semi_utf8_unchecked(decoded)) }
	}

	#[must_use]
	pub fn into_modified_utf8(self) -> Vec<u8> {
		if is_valid_cesu8(&self) {
			self.into_bytes()
		} else {
			self.to_modified_utf8_internal()
		}
	}
}

const fn dec_surrogate(second: u8, third: u8) -> u32 {
	0xd000 | (((second & CONT_MASK) as u32) << 6) | (third & CONT_MASK) as u32
}

fn dec_surrogates(second: u8, third: u8, fifth: u8, sixth: u8) -> [u8; 4] {
	let s1 = dec_surrogate(second, third);
	let s2 = dec_surrogate(fifth, sixth);
	let c = 0x10000 + (((s1 - 0xd800) << 10) | (s2 - 0xdc00));
	assert!((0x01000..=0x0010_ffff).contains(&c));

	[
		0b1111_0000_u8 | ((c & 0b1_1100_0000_0000_0000_0000) >> 18) as u8,
		TAG_CONT | ((c & 0b0_0011_1111_0000_0000_0000) >> 12) as u8,
		TAG_CONT | ((c & 0b0_0000_0000_1111_1100_0000) >> 6) as u8,
		TAG_CONT | (c & 0b0_0000_0000_0000_0011_1111) as u8,
	]
}

fn is_valid_cesu8(text: &JavaStr) -> bool {
	text.bytes()
		.all(|b| !matches!(b, 0) && (matches!(b & !CONT_MASK, TAG_CONT) || utf8_char_width(b) <= 3))
}

const fn enc_surrogate(surrogate: u16) -> [u8; 3] {
	[
		0b1110_0000 | ((surrogate & 0b1111_0000_0000_0000) >> 12) as u8,
		TAG_CONT | ((surrogate & 0b0000_1111_1100_0000) >> 6) as u8,
		TAG_CONT | (surrogate & 0b0000_0000_0011_1111) as u8,
	]
}
