use std::{
	mem,
	ops::{Bound, Range, RangeBounds, RangeTo},
};

use super::{JavaStr, Utf8Error};

pub const TAG_CONT: u8 = 0b1000_0000;
pub const TAG_TWO_B: u8 = 0b1100_0000;
pub const TAG_THREE_B: u8 = 0b1110_0000;
pub const TAG_FOUR_B: u8 = 0b1111_0000;
pub const CONT_MASK: u8 = 0b0011_1111;

const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
	(byte & (0x7f >> width)) as u32
}

const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
	(ch << 6) | (byte & CONT_MASK) as u32
}

const fn utf8_is_cont_byte(byte: u8) -> bool {
	(byte as i8) < -64
}

pub unsafe fn next_code_point<'a, I>(bytes: &mut I) -> Option<u32>
where
	I: Iterator<Item = &'a u8>,
{
	let x = *bytes.next()?;
	if x < 128 {
		return Some(x.into());
	}

	let init = utf8_first_byte(x, 2);
	let y = unsafe { *bytes.next().unwrap_unchecked() };
	let mut ch = utf8_acc_cont_byte(init, y);
	if x >= 0xe0 {
		let z = unsafe { *bytes.next().unwrap_unchecked() };
		let y_z = utf8_acc_cont_byte((y & CONT_MASK).into(), z);
		ch = (init << 12) | y_z;
		if x >= 0xf0 {
			let w = unsafe { *bytes.next().unwrap_unchecked() };
			ch = ((init & 7) << 18) | utf8_acc_cont_byte(y_z, w);
		}
	}

	Some(ch)
}

pub unsafe fn next_code_point_reverse<'a, I>(bytes: &mut I) -> Option<u32>
where
	I: DoubleEndedIterator<Item = &'a u8>,
{
	let w = match *bytes.next_back()? {
		next_byte if next_byte < 128 => return Some(next_byte.into()),
		back_byte => back_byte,
	};

	let mut ch;

	let z = unsafe { *bytes.next_back().unwrap_unchecked() };
	ch = utf8_first_byte(z, 2);

	if utf8_is_cont_byte(z) {
		let y = unsafe { *bytes.next_back().unwrap_unchecked() };
		ch = utf8_first_byte(y, 3);
		if utf8_is_cont_byte(y) {
			let x = unsafe { *bytes.next().unwrap_unchecked() };
			ch = utf8_first_byte(x, 4);
			ch = utf8_acc_cont_byte(ch, y);
		}
		ch = utf8_acc_cont_byte(ch, z);
	}
	ch = utf8_acc_cont_byte(ch, w);

	Some(ch)
}

#[expect(clippy::collapsible_else_if)]
pub fn run_utf8_semi_validation(v: &[u8]) -> Result<(), Utf8Error> {
	let mut index = 0;
	let len = v.len();

	let usize_bytes = mem::size_of::<usize>();
	let ascii_block_size = 2 * usize_bytes;
	let blocks_end = if len >= ascii_block_size {
		len - ascii_block_size + 1
	} else {
		0
	};

	let align = v.as_ptr().align_offset(usize_bytes);

	while index < len {
		let old_offset = index;
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
				index += 1;

				if index >= len {
					err!(::std::option::Option::None)
				}

				v[index]
			}};
		}

		let first = v[index];
		if first >= 128 {
			let w = utf8_char_width(first);

			match w {
				2 => {
					if next!() as i8 >= -64 {
						err!(Some(1))
					}
				}
				3 => {
					match (first, next!()) {
						(0xe0, 0xa0..=0xbf) | (0xe1..=0xef, 0x80..=0xbf) => {}
						_ => err!(Some(1)),
					}
					if next!() as i8 >= -64 {
						err!(Some(2))
					}
				}
				4 => {
					match (first, next!()) {
						(0xf0, 0x90..=0xbf) | (0xf1..=0xf3, 0x80..=0xbf) | (0xf4, 0x80..=0x8f) => {}
						_ => err!(Some(1)),
					}
					if next!() as i8 >= -64 {
						err!(Some(2))
					}
					if next!() as i8 >= -64 {
						err!(Some(3))
					}
				}
				_ => err!(Some(1)),
			}

			index += 1;
		} else {
			if align != usize::MAX && matches!(align.wrapping_sub(index) & usize_bytes, 0) {
				let ptr = v.as_ptr();
				while index < blocks_end {
					unsafe {
						let block = ptr.add(index).cast::<usize>();
						let zu = contains_nonascii(*block);
						let zv = contains_nonascii(*block.add(1));
						if zu || zv {
							break;
						}
					}

					index += ascii_block_size;
				}

				while index < len && v[index] < 128 {
					index += 1;
				}
			} else {
				index += 1;
			}
		}
	}

	Ok(())
}

pub const fn run_utf8_full_validation_from_semi(v: &[u8]) -> Result<(), Utf8Error> {
	let mut index = 0;
	while index + 3 <= v.len() {
		if matches!(v[index], 0xed) && v[index + 1] >= 0xa0 {
			return Err(Utf8Error {
				valid_up_to: index,
				error_len: Some(1),
			});
		}
		index += 1;
	}

	Ok(())
}

pub const fn utf8_char_width(first_byte: u8) -> usize {
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

const fn contains_nonascii(x: usize) -> bool {
	const NONASCII_MASK: usize = usize::from_ne_bytes([0x80; mem::size_of::<usize>()]);
	!matches!(x & NONASCII_MASK, 0)
}

#[cold]
#[track_caller]
pub fn slice_error_fail(s: &JavaStr, begin: usize, end: usize) -> ! {
	const MAX_DISPLAY_LENGTH: usize = 256;
	let trunc_len = s.floor_char_boundary(MAX_DISPLAY_LENGTH);
	let s_trunc = &s[..trunc_len];
	let ellipsis = if trunc_len < s.len() { "[...]" } else { "" };

	if begin > s.len() || end > s.len() {
		let oob_index = if begin > s.len() { begin } else { end };
		panic!("byte index {oob_index} is out of bounds of `{s_trunc}`{ellipsis}");
	}

	assert!(
		begin <= end,
		"begin <= end ({begin} <= {end}) when slicing `{s_trunc}`{ellipsis}"
	);

	let index = if s.is_char_boundary(begin) {
		end
	} else {
		begin
	};

	let char_start = s.floor_char_boundary(index);
	let ch = s[char_start..].chars().next().unwrap();
	let char_range = char_start..char_start + ch.len_utf8();

	panic!(
		"byte index {index} is not a char boundary; is it inside {ch:?} (bytes {char_range:?}) of \
		`{s_trunc}`{ellipsis}"
	);
}

#[cold]
#[track_caller]
pub fn str_end_index_len_fail(index: usize, len: usize) -> ! {
	panic!("range end index {index} out of range for JavaStr of length {len}");
}

#[cold]
#[track_caller]
pub fn str_index_order_fail(index: usize, end: usize) -> ! {
	panic!("JavaStr index starts at {index} but ends at {end}");
}

#[cold]
#[track_caller]
pub fn str_start_index_overflow_fail() -> ! {
	panic!("attempted to index JavaStr from after maximum usize");
}

#[cold]
#[track_caller]
pub fn str_end_index_overflow_fail() -> ! {
	panic!("attempted to index JavaStr up to maximum usize")
}

#[track_caller]
pub fn to_range_checked(range: impl RangeBounds<usize>, bounds: RangeTo<usize>) -> Range<usize> {
	let len = bounds.end;

	let start = range.start_bound();
	let start = match start {
		Bound::Included(&start) => start,
		Bound::Excluded(start) => start
			.checked_add(1)
			.unwrap_or_else(|| str_start_index_overflow_fail()),
		Bound::Unbounded => 0,
	};

	let end = range.end_bound();
	let end = match end {
		Bound::Included(end) => end
			.checked_add(1)
			.unwrap_or_else(|| str_end_index_overflow_fail()),
		Bound::Excluded(&end) => end,
		Bound::Unbounded => len,
	};

	if start > end {
		str_index_order_fail(start, end);
	}

	if end > len {
		str_end_index_len_fail(end, len);
	}

	Range { start, end }
}
