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
	EscapeDebugExtArgs, Utf8Error,
	validations::{run_utf8_full_validation_from_semi, slice_error_fail},
};
use crate::{
	Bytes, CharEscapeIter, CharIndices, Chars, EscapeDebug, EscapeDefault, EscapeUnicode,
	JavaCodePoint, JavaStrPattern, MatchIndices, Matches, RMatchIndices, RMatches, RSplit, RSplitN,
	RSplitTerminator, Split, SplitAsciiWhitespace, validations::str_end_index_overflow_fail,
};

#[repr(transparent)]
pub struct JavaStr {
	inner: [u8],
}

impl JavaStr {
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
}

impl Debug for JavaStr {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		todo!()
	}
}

impl Default for &JavaStr {
	fn default() -> Self {
		JavaStr::from_str("")
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
