use std::{
	char::ParseCharError,
	cmp::Ordering,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hash::{Hash, Hasher},
	io::SeekFrom,
	iter::{FusedIterator, Once, once},
	mem,
	ops::Range,
	str::FromStr,
};

use super::validations::{TAG_CONT, TAG_FOUR_B, TAG_THREE_B, TAG_TWO_B};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct JavaCodePoint {
	#[cfg(target_endian = "little")]
	lower: u16,
	upper: SeventeenValues,
	#[cfg(target_endian = "big")]
	lower: u16,
}

impl JavaCodePoint {
	pub const MAX: Self = Self::from_char(char::MAX);
	pub const REPLACEMENT_CHARACTER: Self = Self::from_char(char::REPLACEMENT_CHARACTER);

	#[must_use]
	pub const fn from_u32(i: u32) -> Option<Self> {
		if i <= 0x0010_ffff {
			unsafe { Some(Self::from_u32_unchecked(i)) }
		} else {
			None
		}
	}

	#[must_use]
	pub const unsafe fn from_u32_unchecked(i: u32) -> Self {
		unsafe { mem::transmute(i) }
	}

	#[must_use]
	pub const fn from_char(c: char) -> Self {
		unsafe { Self::from_u32_unchecked(c as u32) }
	}

	#[must_use]
	pub const fn as_u32(self) -> u32 {
		unsafe {
			let result = mem::transmute::<Self, u32>(self);

			if result > 0x0010_ffff {
				std::hint::unreachable_unchecked();
			}

			result
		}
	}

	#[must_use]
	pub const fn as_char(self) -> Option<char> {
		char::from_u32(self.as_u32())
	}

	#[must_use]
	pub const unsafe fn as_char_unchecked(self) -> char {
		unsafe { char::from_u32_unchecked(self.as_u32()) }
	}

	pub fn encode_utf16(self, dst: &mut [u16]) -> &mut [u16] {
		if let Some(char) = self.as_char() {
			char.encode_utf16(dst)
		} else {
			dst[0] = self.as_u32() as u16;
			&mut dst[..1]
		}
	}

	pub fn encode_semi_utf8(self, dst: &mut [u8]) -> &mut [u8] {
		let len = self.len_utf8();
		let code = self.as_u32();

		match (len, &mut dst[..]) {
			(1, [a, ..]) => {
				*a = code as u8;
			}
			(2, [a, b, ..]) => {
				*a = ((code >> 6) & 0x1f) as u8 | TAG_TWO_B;
				*b = (code & 0x3f) as u8 | TAG_CONT;
			}
			(3, [a, b, c, ..]) => {
				*a = ((code >> 12) & 0x0f) as u8 | TAG_THREE_B;
				*b = ((code >> 6) & 0x3f) as u8 | TAG_CONT;
				*c = (code & 0x3f) as u8 | TAG_CONT;
			}
			(4, [a, b, c, d, ..]) => {
				*a = ((code >> 18) & 0x07) as u8 | TAG_FOUR_B;
				*b = ((code >> 12) & 0x3f) as u8 | TAG_CONT;
				*c = ((code >> 6) & 0x3f) as u8 | TAG_CONT;
				*d = (code & 0x3f) as u8 | TAG_CONT;
			}
			_ => panic!(
				"encode_utf8: need {len} bytes to encode U+{code:X}, but the buffer has {}",
				dst.len()
			),
		}

		&mut dst[..len]
	}

	#[must_use]
	#[expect(clippy::trivially_copy_pass_by_ref)]
	pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
		match (self.as_char(), other.as_char()) {
			(Some(l), Some(r)) => l.eq_ignore_ascii_case(&r),
			(None, None) => self == other,
			_ => false,
		}
	}

	#[must_use]
	pub fn escape_debug(self) -> CharEscapeIter {
		self.escape_debug_ext(EscapeDebugExtArgs::ESCAPE_ALL)
	}

	pub(crate) fn escape_debug_ext(self, args: EscapeDebugExtArgs) -> CharEscapeIter {
		const NULL: u32 = '\0' as u32;
		const TAB: u32 = '\t' as u32;
		const CARRIAGE_RETURN: u32 = '\r' as u32;
		const LINE_FEED: u32 = '\n' as u32;
		const SINGLE_QUOTE: u32 = '\'' as u32;
		const DOUBLE_QUOTE: u32 = '"' as u32;
		const BACKSLASH: u32 = '\\' as u32;

		unsafe {
			match self.as_u32() {
				NULL => CharEscapeIter::new([b'\\', b'0']),
				TAB => CharEscapeIter::new([b'\\', b't']),
				CARRIAGE_RETURN => CharEscapeIter::new([b'\\', b'r']),
				LINE_FEED => CharEscapeIter::new([b'\\', b'n']),
				SINGLE_QUOTE if args.single_quote => CharEscapeIter::new([b'\\', b'\'']),
				DOUBLE_QUOTE if args.double_quote => CharEscapeIter::new([b'\\', b'"']),
				BACKSLASH => CharEscapeIter::new([b'\\', b'\\']),
				_ if self.is_printable() => CharEscapeIter::printable(self.as_char_unchecked()),
				_ => self.escape_unicode(),
			}
		}
	}

	fn is_printable(self) -> bool {
		let Some(char) = self.as_char() else {
			return false;
		};

		if matches!(char, '\\' | '\'' | '"') {
			return true;
		}

		!matches!(char.escape_debug().next(), Some('\\'))
	}

	#[must_use]
	pub fn escape_default(self) -> CharEscapeIter {
		const TAB: u32 = '\t' as u32;
		const CARRIAGE_RETURN: u32 = '\r' as u32;
		const LINE_FEED: u32 = '\n' as u32;
		const SINGLE_QUOTE: u32 = '\'' as u32;
		const DOUBLE_QUOTE: u32 = '"' as u32;
		const BACKSLASH: u32 = '\\' as u32;

		unsafe {
			match self.as_u32() {
				TAB => CharEscapeIter::new([b'\\', b't']),
				CARRIAGE_RETURN => CharEscapeIter::new([b'\\', b'r']),
				LINE_FEED => CharEscapeIter::new([b'\\', b'n']),
				SINGLE_QUOTE => CharEscapeIter::new([b'\\', b'\'']),
				DOUBLE_QUOTE => CharEscapeIter::new([b'\\', b'"']),
				BACKSLASH => CharEscapeIter::new([b'\\', b'\\']),
				0x20..=0x7e => CharEscapeIter::new([self.as_u32() as u8]),
				_ => self.escape_unicode(),
			}
		}
	}

	#[must_use]
	pub fn escape_unicode(self) -> CharEscapeIter {
		let x = self.as_u32();

		let mut arr = [0; 10];
		arr[0] = b'\\';
		arr[1] = b'u';
		arr[2] = b'{';

		let number_len = if matches!(x, 0) {
			1
		} else {
			((x.ilog2() >> 2) + 1) as usize
		};

		arr[3 + number_len] = b'}';
		for hexit in 0..number_len {
			arr[2 + number_len - hexit] = b"0123456789abcdef"[((x >> (hexit << 2)) & 15) as usize];
		}

		CharEscapeIter {
			inner: EscapeIterInner::Escaped(EscapeIterEscaped {
				bytes: arr,
				range: 0..number_len + 4,
			}),
		}
	}

	#[must_use]
	pub fn is_alphabetic(self) -> bool {
		self.as_char().is_some_and(char::is_alphabetic)
	}

	#[must_use]
	pub fn is_alphanumeric(self) -> bool {
		self.as_char().is_some_and(char::is_alphanumeric)
	}

	#[must_use]
	pub const fn is_ascii(self) -> bool {
		self.as_u32() <= 0x7f
	}

	#[must_use]
	pub const fn is_ascii_alphabetic(self) -> bool {
		self.is_ascii_lowercase() || self.is_ascii_uppercase()
	}

	#[must_use]
	pub const fn is_ascii_alphanumeric(self) -> bool {
		self.is_ascii_alphabetic() || self.is_ascii_digit()
	}

	#[must_use]
	pub const fn is_ascii_control(self) -> bool {
		matches!(self.as_u32(), 0..=0x1f | 0x7f)
	}

	#[must_use]
	pub const fn is_ascii_digit(self) -> bool {
		const ZERO: u32 = '0' as u32;
		const NINE: u32 = '9' as u32;
		matches!(self.as_u32(), ZERO..=NINE)
	}

	#[must_use]
	pub const fn is_ascii_graphic(self) -> bool {
		matches!(self.as_u32(), 0x21..=0x7e)
	}

	#[must_use]
	pub const fn is_ascii_hexdigit(self) -> bool {
		const LOWER_A: u32 = 'a' as u32;
		const LOWER_F: u32 = 'f' as u32;
		const UPPER_A: u32 = 'A' as u32;
		const UPPER_F: u32 = 'F' as u32;
		self.is_ascii_digit() || matches!(self.as_u32(), (LOWER_A..=LOWER_F) | (UPPER_A..=UPPER_F))
	}

	#[must_use]
	pub const fn is_ascii_lowercase(self) -> bool {
		const A: u32 = 'a' as u32;
		const Z: u32 = 'z' as u32;
		matches!(self.as_u32(), A..=Z)
	}

	#[must_use]
	pub const fn is_ascii_octdigit(self) -> bool {
		const ZERO: u32 = '0' as u32;
		const SEVEN: u32 = '7' as u32;
		matches!(self.as_u32(), ZERO..=SEVEN)
	}

	#[must_use]
	pub const fn is_ascii_punctuation(self) -> bool {
		matches!(
			self.as_u32(),
			(0x21..=0x2f) | (0x3a..=0x40) | (0x5b..=0x60) | (0x7b..=0x7e)
		)
	}

	#[must_use]
	pub const fn is_ascii_uppercase(self) -> bool {
		const A: u32 = 'A' as u32;
		const Z: u32 = 'Z' as u32;
		matches!(self.as_u32(), A..=Z)
	}

	#[must_use]
	pub const fn is_ascii_whitespace(self) -> bool {
		const SPACE: u32 = ' ' as u32;
		const HORIZONTAL_TAB: u32 = '\t' as u32;
		const LINE_FEED: u32 = '\n' as u32;
		const FORM_FEED: u32 = 0xc;
		const CARRIAGE_RETURN: u32 = '\r' as u32;
		matches!(
			self.as_u32(),
			SPACE | HORIZONTAL_TAB | LINE_FEED | FORM_FEED | CARRIAGE_RETURN
		)
	}

	#[must_use]
	pub fn is_control(self) -> bool {
		self.as_char().is_some_and(char::is_control)
	}

	#[must_use]
	pub const fn is_digit(self, radix: u32) -> bool {
		self.to_digit(radix).is_some()
	}

	#[must_use]
	pub fn is_lowercase(self) -> bool {
		self.as_char().is_some_and(char::is_lowercase)
	}

	#[must_use]
	pub fn is_numeric(self) -> bool {
		self.as_char().is_some_and(char::is_numeric)
	}

	#[must_use]
	pub fn is_uppercase(self) -> bool {
		self.as_char().is_some_and(char::is_uppercase)
	}

	#[must_use]
	pub fn is_whitespace(self) -> bool {
		self.as_char().is_some_and(char::is_whitespace)
	}

	#[must_use]
	pub const fn len_utf16(self) -> usize {
		if let Some(char) = self.as_char() {
			char.len_utf16()
		} else {
			1
		}
	}

	#[must_use]
	pub const fn len_utf8(self) -> usize {
		if let Some(char) = self.as_char() {
			char.len_utf8()
		} else {
			3
		}
	}

	pub const fn make_ascii_lowercase(&mut self) {
		*self = self.to_ascii_lowercase();
	}

	pub const fn make_ascii_uppercase(&mut self) {
		*self = self.to_ascii_uppercase();
	}

	#[must_use]
	pub const fn to_ascii_lowercase(self) -> Self {
		if self.is_ascii_uppercase() {
			unsafe { Self::from_u32_unchecked(self.as_u32() + 32) }
		} else {
			self
		}
	}

	#[must_use]
	pub const fn to_ascii_uppercase(self) -> Self {
		if self.is_ascii_uppercase() {
			unsafe { Self::from_u32_unchecked(self.as_u32() - 32) }
		} else {
			self
		}
	}

	#[must_use]
	pub const fn to_digit(self, radix: u32) -> Option<u32> {
		if let Some(c) = self.as_char() {
			c.to_digit(radix)
		} else {
			None
		}
	}

	#[must_use]
	pub fn to_lowercase(self) -> ToLowercase {
		match self.as_char() {
			Some(ch) => ToLowercase::char(ch.to_lowercase()),
			None => ToLowercase::invalid(self),
		}
	}

	#[must_use]
	pub fn to_uppercase(self) -> ToUppercase {
		match self.as_char() {
			Some(ch) => ToUppercase::char(ch.to_uppercase()),
			None => ToUppercase::invalid(self),
		}
	}
}

impl Debug for JavaCodePoint {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('\'')?;
		for c in self.escape_debug_ext(EscapeDebugExtArgs {
			single_quote: true,
			double_quote: false,
		}) {
			f.write_char(c)?;
		}

		f.write_char('\'')
	}
}

impl Default for JavaCodePoint {
	fn default() -> Self {
		Self::from_char('\0')
	}
}

impl Display for JavaCodePoint {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.as_char().unwrap_or(char::REPLACEMENT_CHARACTER), f)
	}
}

impl From<u8> for JavaCodePoint {
	fn from(value: u8) -> Self {
		Self::from_char(char::from(value))
	}
}

impl From<JavaCodePoint> for u32 {
	fn from(value: JavaCodePoint) -> Self {
		value.as_u32()
	}
}

impl FromStr for JavaCodePoint {
	type Err = ParseCharError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		char::from_str(s).map(Self::from_char)
	}
}

impl Hash for JavaCodePoint {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_u32().hash(state);
	}
}

impl Ord for JavaCodePoint {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_u32().cmp(&other.as_u32())
	}
}

impl PartialEq<char> for JavaCodePoint {
	fn eq(&self, other: &char) -> bool {
		self == &Self::from_char(*other)
	}
}

impl PartialEq<JavaCodePoint> for char {
	fn eq(&self, other: &JavaCodePoint) -> bool {
		other == self
	}
}

impl PartialOrd for JavaCodePoint {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialOrd<char> for JavaCodePoint {
	fn partial_cmp(&self, other: &char) -> Option<Ordering> {
		self.partial_cmp(&Self::from_char(*other))
	}
}

impl PartialOrd<JavaCodePoint> for char {
	fn partial_cmp(&self, other: &JavaCodePoint) -> Option<Ordering> {
		JavaCodePoint::from_char(*self).partial_cmp(other)
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CharIterDelegate<I>(CharIterDelegateInner<I>);

impl<I> CharIterDelegate<I> {
	const fn char(iter: I) -> Self {
		Self(CharIterDelegateInner::Char(iter))
	}

	fn invalid(code_point: JavaCodePoint) -> Self {
		Self(CharIterDelegateInner::Invalid(once(code_point)))
	}
}

impl<I> DoubleEndedIterator for CharIterDelegate<I>
where
	I: DoubleEndedIterator + Iterator<Item = char>,
{
	fn next_back(&mut self) -> Option<Self::Item> {
		match &mut self.0 {
			CharIterDelegateInner::Char(c) => c.next_back().map(JavaCodePoint::from_char),
			CharIterDelegateInner::Invalid(c) => c.next_back(),
		}
	}
}

impl<I> ExactSizeIterator for CharIterDelegate<I> where I: ExactSizeIterator + Iterator<Item = char> {}

impl<I> FusedIterator for CharIterDelegate<I> where I: FusedIterator + Iterator<Item = char> {}

impl<I> Iterator for CharIterDelegate<I>
where
	I: Iterator<Item = char>,
{
	type Item = JavaCodePoint;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.0 {
			CharIterDelegateInner::Char(c) => c.next().map(JavaCodePoint::from_char),
			CharIterDelegateInner::Invalid(code_point) => code_point.next(),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match &self.0 {
			CharIterDelegateInner::Char(c) => c.size_hint(),
			CharIterDelegateInner::Invalid(c) => c.size_hint(),
		}
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CharEscapeIter {
	inner: EscapeIterInner,
}

impl CharEscapeIter {
	fn printable(c: char) -> Self {
		Self {
			inner: EscapeIterInner::Printable(once(c)),
		}
	}

	unsafe fn new<const N: usize>(bytes: [u8; N]) -> Self {
		assert!(N <= 10, "too many bytes in escape iter");
		let mut ten_bytes = [0; 10];
		ten_bytes[..N].copy_from_slice(&bytes);

		Self {
			inner: EscapeIterInner::Escaped(EscapeIterEscaped {
				bytes: ten_bytes,
				range: 0..N,
			}),
		}
	}
}

impl Display for CharEscapeIter {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.inner, f)
	}
}

impl ExactSizeIterator for CharEscapeIter {
	fn len(&self) -> usize {
		match &self.inner {
			EscapeIterInner::Printable(p) => p.len(),
			EscapeIterInner::Escaped(e) => e.len(),
		}
	}
}

impl FusedIterator for CharEscapeIter {}

impl Iterator for CharEscapeIter {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			EscapeIterInner::Printable(p) => p.next(),
			EscapeIterInner::Escaped(e) => e.next(),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match &self.inner {
			EscapeIterInner::Printable(p) => p.size_hint(),
			EscapeIterInner::Escaped(e) => e.size_hint(),
		}
	}
}

pub(crate) struct EscapeDebugExtArgs {
	pub single_quote: bool,
	pub double_quote: bool,
}

impl EscapeDebugExtArgs {
	pub const ESCAPE_ALL: Self = Self {
		single_quote: true,
		double_quote: true,
	};
}

#[derive(Debug, Clone)]
struct EscapeIterEscaped {
	bytes: [u8; 10],
	range: Range<usize>,
}

impl Display for EscapeIterEscaped {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let str =
			unsafe { std::str::from_utf8_unchecked(self.bytes.get_unchecked(self.range.clone())) };

		f.write_str(str)
	}
}

impl ExactSizeIterator for EscapeIterEscaped {
	fn len(&self) -> usize {
		self.range.len()
	}
}

impl FusedIterator for EscapeIterEscaped {}

impl Iterator for EscapeIterEscaped {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		self.range
			.next()
			.map(|index| unsafe { char::from(*self.bytes.get_unchecked(index)) })
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.range.size_hint()
	}

	fn count(self) -> usize {
		self.range.len()
	}
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
enum SeventeenValues {
	V0,
	V1,
	V2,
	V3,
	V4,
	V5,
	V6,
	V7,
	V8,
	V9,
	V10,
	V11,
	V12,
	V13,
	V14,
	V15,
	V16,
}

#[derive(Debug, Clone)]
enum EscapeIterInner {
	Printable(Once<char>),
	Escaped(EscapeIterEscaped),
}

impl Display for EscapeIterInner {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Printable(char) => char.clone().try_for_each(|ch| f.write_char(ch)),
			Self::Escaped(escaped) => Display::fmt(escaped, f),
		}
	}
}

#[derive(Debug, Clone)]
enum CharIterDelegateInner<I> {
	Char(I),
	Invalid(Once<JavaCodePoint>),
}

pub type ToLowercase = CharIterDelegate<std::char::ToLowercase>;
pub type ToUppercase = CharIterDelegate<std::char::ToUppercase>;
