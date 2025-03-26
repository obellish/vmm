mod private {
	use std::ops;

	pub trait Sealed {}

	impl Sealed for ops::Range<usize> {}
	impl Sealed for ops::RangeTo<usize> {}
	impl Sealed for ops::RangeFrom<usize> {}
	impl Sealed for ops::RangeFull {}
	impl Sealed for ops::RangeInclusive<usize> {}
	impl Sealed for ops::RangeToInclusive<usize> {}
}

use std::{
	borrow::Cow,
	collections::Bound,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hash::{Hash, Hasher},
	mem,
	ops::{
		Add, AddAssign, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive,
		RangeTo, RangeToInclusive,
	},
	ptr,
	rc::Rc,
	slice,
	str::FromStr,
	sync::Arc,
};

use super::{
	Bytes, CharEscapeIter, CharIndices, Chars, EscapeDebug, EscapeDebugExtArgs, EscapeDefault,
	EscapeUnicode, JavaCodePoint, JavaStrPattern, JavaString, Lines, MatchIndices, Matches,
	ParseError, RMatchIndices, RMatches, RSplit, RSplitN, RSplitTerminator, Split,
	SplitAsciiWhitespace, SplitInclusive, SplitN, SplitTerminator, SplitWhitespace, Utf8Error,
	validations::{
		run_utf8_full_validation_from_semi, run_utf8_semi_validation, slice_error_fail,
		str_end_index_overflow_fail,
	},
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct JavaStr {
	inner: [u8],
}

impl JavaStr {
	pub const fn from_full_utf8(v: &[u8]) -> Result<&Self, Utf8Error> {
		match std::str::from_utf8(v) {
			Ok(str) => Ok(Self::from_str(str)),
			Err(err) => Err(Utf8Error::from_std(err)),
		}
	}

	pub fn from_full_utf8_mut(v: &mut [u8]) -> Result<&mut Self, Utf8Error> {
		match std::str::from_utf8_mut(v) {
			Ok(str) => Ok(Self::from_mut_str(str)),
			Err(err) => Err(Utf8Error::from_std(err)),
		}
	}

	pub fn from_semi_utf8(v: &[u8]) -> Result<&Self, Utf8Error> {
		run_utf8_semi_validation(v)?;

		Ok(unsafe { Self::from_semi_utf8_unchecked(v) })
	}

	pub fn from_semi_utf8_mut(v: &mut [u8]) -> Result<&mut Self, Utf8Error> {
		run_utf8_semi_validation(v)?;

		Ok(unsafe { Self::from_semi_utf8_unchecked_mut(v) })
	}

	#[must_use]
	pub const unsafe fn from_semi_utf8_unchecked(v: &[u8]) -> &Self {
		unsafe { mem::transmute(v) }
	}

	pub unsafe fn from_semi_utf8_unchecked_mut(v: &mut [u8]) -> &mut Self {
		unsafe { &mut *(ptr::from_mut::<[u8]>(v) as *mut Self) }
	}

	#[must_use]
	pub const fn from_str(s: &str) -> &Self {
		unsafe { Self::from_semi_utf8_unchecked(s.as_bytes()) }
	}

	pub fn from_mut_str(s: &mut str) -> &mut Self {
		unsafe { Self::from_semi_utf8_unchecked_mut(s.as_bytes_mut()) }
	}

	#[must_use]
	pub fn from_boxed_str(v: Box<str>) -> Box<Self> {
		unsafe { Self::from_boxed_semi_utf8_unchecked(v.into_boxed_bytes()) }
	}

	#[must_use]
	pub unsafe fn from_boxed_semi_utf8_unchecked(v: Box<[u8]>) -> Box<Self> {
		unsafe { Box::from_raw(Box::into_raw(v) as *mut Self) }
	}

	#[must_use]
	pub const fn as_bytes(&self) -> &[u8] {
		&self.inner
	}

	pub const unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.inner
	}

	pub const fn as_mut_ptr(&mut self) -> *mut u8 {
		self.inner.as_mut_ptr()
	}

	#[must_use]
	pub const fn as_ptr(&self) -> *const u8 {
		self.inner.as_ptr()
	}

	pub const fn as_str(&self) -> Result<&str, Utf8Error> {
		match run_utf8_full_validation_from_semi(self.as_bytes()) {
			Ok(..) => unsafe { Ok(self.as_str_unchecked()) },
			Err(e) => Err(e),
		}
	}

	#[must_use]
	pub const unsafe fn as_str_unchecked(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	#[must_use]
	pub fn as_str_lossy(&self) -> Cow<'_, str> {
		match run_utf8_full_validation_from_semi(self.as_bytes()) {
			Ok(()) => unsafe { Cow::Borrowed(self.as_str_unchecked()) },
			Err(error) => unsafe {
				Cow::Owned(
					self.transform_invalid_string(error, str::to_owned, |_| {
						Self::from_str("\u{FFFD}")
					})
					.into_string_unchecked(),
				)
			},
		}
	}

	#[must_use]
	pub fn eq_ignore_ascii_case(&self, other: &str) -> bool {
		self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
	}

	#[must_use]
	pub fn eq_java_ignore_ascii_case(&self, other: &Self) -> bool {
		self.as_bytes().eq_ignore_ascii_case(other.as_bytes())
	}

	pub fn get(&self, i: impl JavaStrSliceIndex) -> Option<&Self> {
		i.get(self)
	}

	pub fn get_mut(&mut self, i: impl JavaStrSliceIndex) -> Option<&mut Self> {
		i.get_mut(self)
	}

	pub unsafe fn get_unchecked(&self, i: impl JavaStrSliceIndex) -> &Self {
		unsafe { &*i.get_unchecked(self) }
	}

	pub unsafe fn get_unchecked_mut(&mut self, i: impl JavaStrSliceIndex) -> &mut Self {
		unsafe { &mut *i.get_unchecked_mut(self) }
	}

	#[must_use]
	pub fn into_boxed_bytes(self: Box<Self>) -> Box<[u8]> {
		unsafe { Box::from_raw(Box::into_raw(self) as *mut [u8]) }
	}

	#[must_use]
	pub fn is_char_boundary(&self, index: usize) -> bool {
		if matches!(index, 0) {
			return true;
		}

		match self.as_bytes().get(index) {
			None => index == self.len(),
			Some(&b) => (b as i8) >= -0x40,
		}
	}

	#[must_use]
	pub const fn len(&self) -> usize {
		self.inner.len()
	}

	#[must_use]
	pub const fn is_empty(&self) -> bool {
		matches!(self.len(), 0)
	}

	pub fn bytes(&self) -> Bytes<'_> {
		Bytes {
			inner: self.inner.iter().copied(),
		}
	}

	pub fn char_indices(&self) -> CharIndices<'_> {
		CharIndices {
			front_offset: 0,
			inner: self.chars(),
		}
	}

	pub fn chars(&self) -> Chars<'_> {
		Chars {
			inner: self.inner.iter(),
		}
	}

	pub fn contains(&self, mut pat: impl JavaStrPattern) -> bool {
		pat.find_in(self).is_some()
	}

	pub fn ends_with(&self, mut pat: impl JavaStrPattern) -> bool {
		pat.suffix_len_in(self).is_some()
	}

	pub fn escape_debug(&self) -> EscapeDebug<'_> {
		fn escape_first(first: JavaCodePoint) -> CharEscapeIter {
			first.escape_debug()
		}

		fn escape_rest(char: JavaCodePoint) -> CharEscapeIter {
			char.escape_debug()
		}

		let mut chars = self.chars();
		EscapeDebug {
			inner: chars
				.next()
				.map(escape_first as fn(JavaCodePoint) -> CharEscapeIter)
				.into_iter()
				.flatten()
				.chain(chars.flat_map(escape_rest as fn(JavaCodePoint) -> CharEscapeIter)),
		}
	}

	pub fn escape_default(&self) -> EscapeDefault<'_> {
		EscapeDefault {
			inner: self.chars().flat_map(JavaCodePoint::escape_default),
		}
	}

	pub fn escape_unicode(&self) -> EscapeUnicode<'_> {
		EscapeUnicode {
			inner: self.chars().flat_map(JavaCodePoint::escape_unicode),
		}
	}

	pub fn find(&self, mut pat: impl JavaStrPattern) -> Option<usize> {
		pat.find_in(self).map(|(index, ..)| index)
	}

	#[must_use]
	pub const fn is_ascii(&self) -> bool {
		self.as_bytes().is_ascii()
	}

	pub(crate) fn floor_char_boundary(&self, index: usize) -> usize {
		if index >= self.len() {
			self.len()
		} else {
			let lower_bound = index.saturating_sub(3);
			let new_index = self.as_bytes()[lower_bound..=index]
				.iter()
				.rposition(|b| (*b as i8) >= -0x40);

			unsafe { lower_bound + new_index.unwrap_unchecked() }
		}
	}

	pub const fn make_ascii_lowercase(&mut self) {
		let me = unsafe { self.as_bytes_mut() };
		me.make_ascii_lowercase();
	}

	pub const fn make_ascii_uppercase(&mut self) {
		let me = unsafe { self.as_bytes_mut() };
		me.make_ascii_uppercase();
	}

	pub const fn match_indices<P>(&self, pat: P) -> MatchIndices<'_, P> {
		MatchIndices {
			str: self,
			start: 0,
			pat,
		}
	}

	pub const fn matches<P>(&self, pat: P) -> Matches<'_, P> {
		Matches { str: self, pat }
	}

	pub fn rfind(&self, mut pat: impl JavaStrPattern) -> Option<usize> {
		pat.rfind_in(self).map(|(index, ..)| index)
	}

	pub const fn rmatch_indices<P>(&self, pat: P) -> RMatchIndices<'_, P> {
		RMatchIndices {
			inner: self.match_indices(pat),
		}
	}

	pub const fn rmatches<P>(&self, pat: P) -> RMatches<'_, P> {
		RMatches {
			inner: self.matches(pat),
		}
	}

	pub fn starts_with(&self, mut pat: impl JavaStrPattern) -> bool {
		pat.prefix_len_in(self).is_some()
	}

	pub fn strip_prefix(&self, mut pat: impl JavaStrPattern) -> Option<&Self> {
		let len = pat.prefix_len_in(self)?;
		unsafe { Some(self.get_unchecked(len..)) }
	}

	pub fn strip_suffix(&self, mut pat: impl JavaStrPattern) -> Option<&Self> {
		let len = pat.suffix_len_in(self)?;
		unsafe { Some(self.get_unchecked(..self.len() - len)) }
	}

	pub fn trim(&self) -> &Self {
		self.trim_matches(JavaCodePoint::is_whitespace)
	}

	#[must_use]
	pub fn trim_end(&self) -> &Self {
		self.trim_end_matches(JavaCodePoint::is_whitespace)
	}

	pub fn trim_end_matches(&self, mut pat: impl JavaStrPattern) -> &Self {
		let mut str = self;
		while let Some(suffix_len) = pat.suffix_len_in(str) {
			if matches!(suffix_len, 0) {
				break;
			}

			str = unsafe { str.get_unchecked(..str.len() - suffix_len) };
		}

		str
	}

	pub fn trim_matches(&self, mut pat: impl JavaStrPattern) -> &Self {
		let mut str = self;
		while let Some(prefix_len) = pat.prefix_len_in(str) {
			if matches!(prefix_len, 0) {
				break;
			}

			str = unsafe { str.get_unchecked(prefix_len..) };
		}

		while let Some(suffix_len) = pat.suffix_len_in(str) {
			if matches!(suffix_len, 0) {
				break;
			}

			str = unsafe { str.get_unchecked(..str.len() - suffix_len) };
		}

		str
	}

	#[must_use]
	pub fn trim_start(&self) -> &Self {
		self.trim_start_matches(JavaCodePoint::is_whitespace)
	}

	pub fn trim_start_matches(&self, mut pat: impl JavaStrPattern) -> &Self {
		let mut str = self;
		while let Some(prefix_len) = pat.prefix_len_in(str) {
			if matches!(prefix_len, 0) {
				break;
			}

			str = unsafe { str.get_unchecked(prefix_len..) };
		}

		str
	}

	pub const fn rsplit<P: JavaStrPattern>(&self, pat: P) -> RSplit<'_, P> {
		RSplit::new(self, pat)
	}

	pub fn rsplit_once(&self, mut delimiter: impl JavaStrPattern) -> Option<(&Self, &Self)> {
		let (index, len) = delimiter.rfind_in(self)?;
		unsafe {
			Some((
				self.get_unchecked(..index),
				self.get_unchecked(index + len..),
			))
		}
	}

	pub const fn rsplit_terminator<P: JavaStrPattern>(&self, pat: P) -> RSplitTerminator<'_, P> {
		RSplitTerminator::new(self, pat)
	}

	pub const fn rsplitn<P: JavaStrPattern>(&self, n: usize, pat: P) -> RSplitN<'_, P> {
		RSplitN::new(self, pat, n)
	}

	pub const fn split<P: JavaStrPattern>(&self, pat: P) -> Split<'_, P> {
		Split::new(self, pat)
	}

	#[must_use]
	pub fn split_ascii_whitespace(&self) -> SplitAsciiWhitespace<'_> {
		#[expect(clippy::trivially_copy_pass_by_ref)]
		const fn is_non_empty(bytes: &&[u8]) -> bool {
			!bytes.is_empty()
		}

		SplitAsciiWhitespace {
			inner: self
				.as_bytes()
				.split(u8::is_ascii_whitespace as fn(&u8) -> bool)
				.filter(is_non_empty as fn(&&[u8]) -> bool)
				.map(|bytes| unsafe { Self::from_semi_utf8_unchecked(bytes) }),
		}
	}

	#[must_use]
	pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
		if self.is_char_boundary(mid) {
			unsafe {
				(
					self.get_unchecked(0..mid),
					self.get_unchecked(mid..self.len()),
				)
			}
		} else {
			slice_error_fail(self, 0, mid)
		}
	}

	pub fn split_at_mut(&mut self, mid: usize) -> (&mut Self, &mut Self) {
		if self.is_char_boundary(mid) {
			let len = self.len();
			let ptr = self.as_mut_ptr();

			unsafe {
				(
					Self::from_semi_utf8_unchecked_mut(slice::from_raw_parts_mut(ptr, mid)),
					Self::from_semi_utf8_unchecked_mut(slice::from_raw_parts_mut(
						ptr.add(mid),
						len - mid,
					)),
				)
			}
		} else {
			slice_error_fail(self, 0, mid)
		}
	}

	pub const fn split_inclusive<P: JavaStrPattern>(&self, pat: P) -> SplitInclusive<'_, P> {
		SplitInclusive::new(self, pat)
	}

	pub fn split_once(&self, mut delimiter: impl JavaStrPattern) -> Option<(&Self, &Self)> {
		let (index, len) = delimiter.find_in(self)?;

		unsafe {
			Some((
				self.get_unchecked(..index),
				self.get_unchecked(index + len..),
			))
		}
	}

	pub const fn split_terminator<P: JavaStrPattern>(&self, pat: P) -> SplitTerminator<'_, P> {
		SplitTerminator::new(self, pat)
	}

	#[must_use]
	pub fn split_whitespace(&self) -> SplitWhitespace<'_> {
		SplitWhitespace {
			inner: self
				.split(JavaCodePoint::is_whitespace as fn(JavaCodePoint) -> bool)
				.filter(|str| !str.is_empty()),
		}
	}

	pub const fn splitn<P: JavaStrPattern>(&self, n: usize, pat: P) -> SplitN<'_, P> {
		SplitN::new(self, pat, n)
	}

	#[must_use]
	pub fn into_string(self: Box<Self>) -> JavaString {
		let slice = self.into_boxed_bytes();
		unsafe { JavaString::from_semi_utf8_unchecked(slice.into_vec()) }
	}

	pub fn lines(&self) -> Lines<'_> {
		Lines {
			inner: self.split_inclusive('\n').map(|line| {
				let Some(line) = line.strip_suffix('\n') else {
					return line;
				};

				let Some(line) = line.strip_suffix('\r') else {
					return line;
				};

				line
			}),
		}
	}

	#[must_use]
	pub fn repeat(&self, n: usize) -> JavaString {
		unsafe { JavaString::from_semi_utf8_unchecked(self.as_bytes().repeat(n)) }
	}

	pub fn replace(&self, from: impl JavaStrPattern, to: &str) -> JavaString {
		self.replace_java(from, Self::from_str(to))
	}

	pub fn replace_java(&self, from: impl JavaStrPattern, to: &Self) -> JavaString {
		let mut result = JavaString::new();
		let mut last_end = 0;
		for (start, part) in self.match_indices(from) {
			result.push_java_str(unsafe { self.get_unchecked(last_end..start) });
			result.push_java_str(to);
			last_end = start + part.len();
		}

		result.push_java_str(unsafe { self.get_unchecked(last_end..self.len()) });

		result
	}

	pub fn replacen(&self, from: impl JavaStrPattern, to: &str, count: usize) -> JavaString {
		self.replacen_java(from, Self::from_str(to), count)
	}

	pub fn replacen_java(&self, from: impl JavaStrPattern, to: &Self, count: usize) -> JavaString {
		let mut result = JavaString::with_capacity(32);
		let mut last_end = 0;
		for (start, part) in self.match_indices(from).take(count) {
			result.push_java_str(unsafe { self.get_unchecked(last_end..start) });
			result.push_java_str(to);
			last_end = start + part.len();
		}

		result.push_java_str(unsafe { self.get_unchecked(last_end..self.len()) });
		result
	}

	pub fn parse<F: FromStr>(&self) -> Result<F, ParseError<<F as FromStr>::Err>> {
		let s = self.as_str()?;
		s.parse().map_err(ParseError::Other)
	}

	#[must_use]
	pub fn to_ascii_lowercase(&self) -> JavaString {
		let mut s = self.to_owned();
		s.make_ascii_lowercase();
		s
	}

	#[must_use]
	pub fn to_ascii_uppercase(&self) -> JavaString {
		let mut s = self.to_owned();
		s.make_ascii_uppercase();
		s
	}

	pub fn to_lowercase(&self) -> JavaString {
		self.transform_string(str::to_lowercase, |ch| ch)
	}

	pub fn to_uppercase(&self) -> JavaString {
		self.transform_string(str::to_uppercase, |ch| ch)
	}

	fn transform_string(
		&self,
		mut string_transformer: impl FnMut(&str) -> String,
		invalid_char_transformer: impl FnMut(&Self) -> &Self,
	) -> JavaString {
		let bytes = self.as_bytes();
		match run_utf8_full_validation_from_semi(bytes) {
			Ok(()) => JavaString::from(string_transformer(unsafe {
				std::str::from_utf8_unchecked(bytes)
			})),
			Err(error) => {
				self.transform_invalid_string(error, string_transformer, invalid_char_transformer)
			}
		}
	}

	fn transform_invalid_string(
		&self,
		error: Utf8Error,
		mut string_transformer: impl FnMut(&str) -> String,
		mut invalid_char_transformer: impl FnMut(&Self) -> &Self,
	) -> JavaString {
		let bytes = self.as_bytes();
		let mut result = JavaString::from(string_transformer(unsafe {
			std::str::from_utf8_unchecked(bytes.get_unchecked(..error.valid_up_to))
		}));
		result.push_java_str(invalid_char_transformer(unsafe {
			Self::from_semi_utf8_unchecked(
				bytes.get_unchecked(error.valid_up_to..error.valid_up_to + 3),
			)
		}));
		let mut index = error.valid_up_to + 3;
		loop {
			let remainder = unsafe { bytes.get_unchecked(index..) };
			match run_utf8_full_validation_from_semi(remainder) {
				Ok(()) => {
					result.push_str(&string_transformer(unsafe {
						std::str::from_utf8_unchecked(remainder)
					}));
					return result;
				}
				Err(error) => {
					result.push_str(&string_transformer(unsafe {
						std::str::from_utf8_unchecked(
							bytes.get_unchecked(index..index + error.valid_up_to),
						)
					}));
					result.push_java_str(invalid_char_transformer(unsafe {
						Self::from_semi_utf8_unchecked(bytes.get_unchecked(
							index + error.valid_up_to..index + error.valid_up_to + 3,
						))
					}));
					index += error.valid_up_to + 3;
				}
			}
		}
	}
}

impl Add<&JavaStr> for Cow<'_, JavaStr> {
	type Output = Self;

	fn add(mut self, rhs: &JavaStr) -> Self::Output {
		self += rhs;
		self
	}
}

impl AddAssign<&JavaStr> for Cow<'_, JavaStr> {
	fn add_assign(&mut self, rhs: &JavaStr) {
		if !rhs.is_empty() {
			self.to_mut().push_java_str(rhs);
		}
	}
}

impl AsRef<[u8]> for JavaStr {
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl AsRef<JavaStr> for str {
	fn as_ref(&self) -> &JavaStr {
		JavaStr::from_str(self)
	}
}

impl AsRef<JavaStr> for String {
	fn as_ref(&self) -> &JavaStr {
		JavaStr::from_str(self)
	}
}

impl AsRef<Self> for JavaStr {
	fn as_ref(&self) -> &Self {
		self
	}
}

impl Clone for Box<JavaStr> {
	fn clone(&self) -> Self {
		let buf: Box<[u8]> = self.as_bytes().into();
		unsafe { JavaStr::from_boxed_semi_utf8_unchecked(buf) }
	}
}

impl Debug for JavaStr {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('"')?;
		let mut from = 0;
		for (i, c) in self.char_indices() {
			let esc = c.escape_debug_ext(EscapeDebugExtArgs {
				single_quote: false,
				double_quote: true,
			});

			if esc.len() != 1 || c.as_char().is_none() {
				unsafe {
					f.write_str(self[from..i].as_str_unchecked())?;
				}

				for c in esc {
					f.write_char(c)?;
				}

				from = i + c.len_utf8();
			}
		}

		unsafe {
			f.write_str(self[from..].as_str_unchecked())?;
		}

		f.write_char('"')
	}
}

impl Default for &JavaStr {
	fn default() -> Self {
		JavaStr::from_str("")
	}
}

impl Default for Box<JavaStr> {
	fn default() -> Self {
		JavaStr::from_boxed_str(Box::<str>::default())
	}
}

impl Display for JavaStr {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.as_str_lossy())
	}
}

impl<'a> From<&'a JavaStr> for Cow<'a, JavaStr> {
	fn from(value: &'a JavaStr) -> Self {
		Self::Borrowed(value)
	}
}

impl From<&JavaStr> for Arc<JavaStr> {
	fn from(value: &JavaStr) -> Self {
		let arc = Arc::<[u8]>::from(value.as_bytes());
		unsafe { Self::from_raw(Arc::into_raw(arc) as *const JavaStr) }
	}
}

impl From<&JavaStr> for Box<JavaStr> {
	fn from(value: &JavaStr) -> Self {
		unsafe { JavaStr::from_boxed_semi_utf8_unchecked(Box::from(value.as_bytes())) }
	}
}

impl From<&JavaStr> for Rc<JavaStr> {
	fn from(value: &JavaStr) -> Self {
		let rc = Rc::<[u8]>::from(value.as_bytes());
		unsafe { Self::from_raw(Rc::into_raw(rc) as *const JavaStr) }
	}
}

impl From<&JavaStr> for Vec<u8> {
	fn from(value: &JavaStr) -> Self {
		From::from(value.as_bytes())
	}
}

impl From<Cow<'_, JavaStr>> for Box<JavaStr> {
	fn from(value: Cow<'_, JavaStr>) -> Self {
		match value {
			Cow::Borrowed(s) => Self::from(s),
			Cow::Owned(s) => Self::from(s),
		}
	}
}

impl From<JavaString> for Box<JavaStr> {
	fn from(value: JavaString) -> Self {
		value.into_boxed_str()
	}
}

impl<'a> From<&'a str> for &'a JavaStr {
	fn from(value: &'a str) -> Self {
		JavaStr::from_str(value)
	}
}

impl<'a> From<&'a String> for &'a JavaStr {
	fn from(value: &'a String) -> Self {
		JavaStr::from_str(value)
	}
}

impl Hash for JavaStr {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write(self.as_bytes());
		state.write_u8(0xff);
	}
}

impl<I: JavaStrSliceIndex> Index<I> for JavaStr {
	type Output = Self;

	fn index(&self, index: I) -> &Self::Output {
		index.index(self)
	}
}

impl<I: JavaStrSliceIndex> IndexMut<I> for JavaStr {
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		index.index_mut(self)
	}
}

impl<'b> PartialEq<&'b JavaStr> for Cow<'_, str> {
	fn eq(&self, other: &&'b JavaStr) -> bool {
		self == *other
	}
}

impl<'b> PartialEq<&'b JavaStr> for Cow<'_, JavaStr> {
	fn eq(&self, other: &&'b JavaStr) -> bool {
		self == *other
	}
}

impl<'a> PartialEq<Cow<'a, str>> for &JavaStr {
	fn eq(&self, other: &Cow<'a, str>) -> bool {
		*self == other
	}
}

impl<'a> PartialEq<Cow<'a, str>> for JavaStr {
	fn eq(&self, other: &Cow<'a, str>) -> bool {
		other == self
	}
}

impl<'a> PartialEq<Cow<'a, JavaStr>> for &JavaStr {
	fn eq(&self, other: &Cow<'a, JavaStr>) -> bool {
		*self == other
	}
}

impl<'a> PartialEq<Cow<'a, Self>> for JavaStr {
	fn eq(&self, other: &Cow<'a, Self>) -> bool {
		other == self
	}
}

impl PartialEq<String> for &JavaStr {
	fn eq(&self, other: &String) -> bool {
		*self == other
	}
}

impl PartialEq<String> for JavaStr {
	fn eq(&self, other: &String) -> bool {
		self == &other[..]
	}
}

impl PartialEq<JavaStr> for String {
	fn eq(&self, other: &JavaStr) -> bool {
		&self[..] == other
	}
}

impl PartialEq<JavaString> for &JavaStr {
	fn eq(&self, other: &JavaString) -> bool {
		*self == other
	}
}

impl PartialEq<JavaString> for JavaStr {
	fn eq(&self, other: &JavaString) -> bool {
		self == other[..]
	}
}

impl PartialEq<JavaStr> for Cow<'_, str> {
	fn eq(&self, other: &JavaStr) -> bool {
		match self {
			Cow::Borrowed(this) => this == other,
			Cow::Owned(this) => this == other,
		}
	}
}

impl PartialEq<JavaStr> for Cow<'_, JavaStr> {
	fn eq(&self, other: &JavaStr) -> bool {
		match self {
			Self::Borrowed(this) => this == other,
			Self::Owned(this) => this == other,
		}
	}
}

impl PartialEq<JavaStr> for str {
	fn eq(&self, other: &JavaStr) -> bool {
		JavaStr::from_str(self) == other
	}
}

impl PartialEq<JavaStr> for &str {
	fn eq(&self, other: &JavaStr) -> bool {
		self.as_bytes() == &other.inner
	}
}

impl PartialEq<str> for JavaStr {
	fn eq(&self, other: &str) -> bool {
		&self.inner == other.as_bytes()
	}
}

impl<'a> PartialEq<&'a str> for JavaStr {
	fn eq(&self, other: &&'a str) -> bool {
		&self.inner == other.as_bytes()
	}
}

impl PartialEq<JavaStr> for &JavaStr {
	fn eq(&self, other: &JavaStr) -> bool {
		self.inner == other.inner
	}
}

impl<'a> PartialEq<&'a Self> for JavaStr {
	fn eq(&self, other: &&'a Self) -> bool {
		self.inner == other.inner
	}
}

impl ToOwned for JavaStr {
	type Owned = JavaString;

	fn to_owned(&self) -> Self::Owned {
		unsafe { JavaString::from_semi_utf8_unchecked(self.as_bytes().to_owned()) }
	}
}

pub unsafe trait JavaStrSliceIndex: self::private::Sealed + Sized {
	fn check_bounds(&self, slice: &JavaStr) -> bool;
	fn check_bounds_fail(self, slice: &JavaStr) -> !;

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr;

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr;

	fn get(self, slice: &JavaStr) -> Option<&JavaStr> {
		self.check_bounds(slice)
			.then(|| unsafe { &*self.get_unchecked(slice) })
	}

	fn get_mut(self, slice: &mut JavaStr) -> Option<&mut JavaStr> {
		self.check_bounds(slice)
			.then(|| unsafe { &mut *self.get_unchecked_mut(slice) })
	}

	fn index(self, slice: &JavaStr) -> &JavaStr {
		if self.check_bounds(slice) {
			unsafe { &*self.get_unchecked(slice) }
		} else {
			self.check_bounds_fail(slice)
		}
	}

	fn index_mut(self, slice: &mut JavaStr) -> &mut JavaStr {
		if self.check_bounds(slice) {
			unsafe { &mut *self.get_unchecked_mut(slice) }
		} else {
			self.check_bounds_fail(slice)
		}
	}
}

unsafe impl JavaStrSliceIndex for RangeFull {
	fn check_bounds(&self, _: &JavaStr) -> bool {
		true
	}

	fn check_bounds_fail(self, _: &JavaStr) -> ! {
		unreachable!()
	}

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr {
		slice
	}

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr {
		slice
	}
}

unsafe impl JavaStrSliceIndex for Range<usize> {
	fn check_bounds(&self, slice: &JavaStr) -> bool {
		self.start <= self.end
			&& slice.is_char_boundary(self.start)
			&& slice.is_char_boundary(self.end)
	}

	fn check_bounds_fail(self, slice: &JavaStr) -> ! {
		slice_error_fail(slice, self.start, self.end)
	}

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr {
		let slice = slice as *const [u8];
		let ptr = unsafe { slice.cast::<u8>().add(self.start) };
		let len = self.end - self.start;
		ptr::slice_from_raw_parts(ptr, len) as *const JavaStr
	}

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr {
		let slice = slice as *mut [u8];
		let ptr = unsafe { slice.cast::<u8>().add(self.start) };
		let len = self.end - self.start;
		ptr::slice_from_raw_parts_mut(ptr, len) as *mut JavaStr
	}
}

unsafe impl JavaStrSliceIndex for RangeTo<usize> {
	fn check_bounds(&self, slice: &JavaStr) -> bool {
		slice.is_char_boundary(self.end)
	}

	fn check_bounds_fail(self, slice: &JavaStr) -> ! {
		slice_error_fail(slice, 0, self.end)
	}

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr {
		unsafe { (0..self.end).get_unchecked(slice) }
	}

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr {
		unsafe { (0..self.end).get_unchecked_mut(slice) }
	}
}

unsafe impl JavaStrSliceIndex for RangeFrom<usize> {
	fn check_bounds(&self, slice: &JavaStr) -> bool {
		slice.is_char_boundary(self.start)
	}

	fn check_bounds_fail(self, slice: &JavaStr) -> ! {
		slice_error_fail(slice, self.start, slice.len())
	}

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr {
		let len = unsafe { (*(slice as *const [u8])).len() };
		unsafe { (self.start..len).get_unchecked(slice) }
	}

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr {
		let len = unsafe { (*(slice as *mut [u8])).len() };
		unsafe { (self.start..len).get_unchecked_mut(slice) }
	}
}

unsafe impl JavaStrSliceIndex for RangeInclusive<usize> {
	fn check_bounds(&self, slice: &JavaStr) -> bool {
		*self.end() != usize::MAX && into_slice_range(self.clone()).check_bounds(slice)
	}

	fn check_bounds_fail(self, slice: &JavaStr) -> ! {
		if *self.end() == usize::MAX {
			str_end_index_overflow_fail()
		} else {
			into_slice_range(self).check_bounds_fail(slice)
		}
	}

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr {
		unsafe { into_slice_range(self).get_unchecked(slice) }
	}

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr {
		unsafe { into_slice_range(self).get_unchecked_mut(slice) }
	}
}

unsafe impl JavaStrSliceIndex for RangeToInclusive<usize> {
	fn check_bounds(&self, slice: &JavaStr) -> bool {
		(0..=self.end).check_bounds(slice)
	}

	fn check_bounds_fail(self, slice: &JavaStr) -> ! {
		(0..=self.end).check_bounds_fail(slice)
	}

	unsafe fn get_unchecked(self, slice: *const JavaStr) -> *const JavaStr {
		unsafe { (0..=self.end).get_unchecked(slice) }
	}

	unsafe fn get_unchecked_mut(self, slice: *mut JavaStr) -> *mut JavaStr {
		unsafe { (0..=self.end).get_unchecked_mut(slice) }
	}
}

fn into_slice_range(range: RangeInclusive<usize>) -> Range<usize> {
	let exclusive_end = *range.end() + 1;
	let start = match range.end_bound() {
		Bound::Excluded(..) => exclusive_end,
		Bound::Included(..) => *range.start(),
		Bound::Unbounded => unreachable!(),
	};

	start..exclusive_end
}
