#![expect(clippy::many_single_char_names)]

use std::{
	io::{Result as IoResult, prelude::*},
	str::from_utf8_unchecked,
};

use byteorder::{BigEndian, WriteBytesExt as _};

pub fn write_modified_utf8(mut writer: impl Write, text: &str) -> IoResult<()> {
	let bytes = text.as_bytes();
	let mut i = 0;

	while i < bytes.len() {
		match bytes[i] {
			0 => {
				writer.write_u16::<BigEndian>(0xc080)?;
				i += 1;
			}
			b @ 1..=127 => {
				writer.write_u8(b)?;
				i += 1;
			}
			b => {
				let w = utf8_char_width(b);
				debug_assert!(w <= 4);
				debug_assert!(i + w <= bytes.len());

				if matches!(w, 4) {
					let s = unsafe { from_utf8_unchecked(&bytes[i..i + w]) };
					let c = s.chars().next().unwrap() as u32 - 0x10000;

					let s0 = ((c >> 10) as u16) | 0xd800;
					let s1 = ((c & 0x3ff) as u16) | 0xdc00;

					writer.write_all(encode_surrogate(s0).as_slice())?;
					writer.write_all(encode_surrogate(s1).as_slice())?;
				} else {
					writer.write_all(&bytes[i..i + w])?;
				}
				i += w;
			}
		}
	}

	Ok(())
}

pub fn encoded_len(bytes: &[u8]) -> usize {
	let mut n = 0;
	let mut i = 0;

	while i < bytes.len() {
		match bytes[i] {
			1..=127 => {
				n += 1;
				i += 1;
			}
			0 => {
				n += 2;
				i += 1;
			}
			b => {
				let w = utf8_char_width(b);

				if matches!(w, 4) {
					n += 6;
				} else {
					n += w;
				}

				i += w;
			}
		}
	}

	n
}

const fn utf8_char_width(first_byte: u8) -> usize {
	const UTF8_CHAR_WIDTH: [u8; 256] = [
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
		2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
		4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	];

	UTF8_CHAR_WIDTH[first_byte as usize] as usize
}

fn encode_surrogate(surrogate: u16) -> [u8; 3] {
	const TAG_CONT_U8: u8 = 0b1000_0000;
	debug_assert!((0xd800..=0xdfff).contains(&surrogate));
	[
		0b1110_0000 | ((surrogate & 0b1111_0000_0000_0000) >> 12) as u8,
		TAG_CONT_U8 | ((surrogate & 0b0000_1111_1100_0000) >> 6) as u8,
		TAG_CONT_U8 | (surrogate & 0b0000_0000_0011_1111) as u8,
	]
}

#[cfg(test)]
mod tests {
	use anyhow::Result;

	use super::{encoded_len, write_modified_utf8};

	fn check(s: &str) -> Result<()> {
		let mut ours = Vec::new();

		let theirs = cesu8::to_java_cesu8(s);
		write_modified_utf8(&mut ours, s)?;

		assert_eq!(theirs, ours);
		assert_eq!(theirs.len(), encoded_len(s.as_bytes()));

		Ok(())
	}

	#[test]
	fn equivalence() -> Result<()> {
		check("Mary had a little lamb\0")?;
		check("🤡💩👻💀☠👽👾🤖🎃😺😸😹😻😼😽🙀😿😾")?;
		check("ÅÆÇÈØõ÷£¥ý")?;

		Ok(())
	}
}
