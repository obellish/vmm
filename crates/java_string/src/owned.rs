use std::{
	borrow::{Borrow, BorrowMut, Cow},
	collections::{Bound, TryReserveError},
	convert::Infallible,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write},
	hash::{Hash, Hasher},
	iter::FusedIterator,
	ops::{
		Add, AddAssign, Deref, DerefMut, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull,
		RangeInclusive, RangeTo, RangeToInclusive,
	},
	ptr,
	rc::Rc,
	slice,
	str::FromStr,
	sync::Arc,
};

use super::{
	Chars, FromUtf8Error, JavaCodePoint, JavaStr, Utf8Error,
	validations::{run_utf8_full_validation_from_semi, run_utf8_semi_validation, to_range_checked},
};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct JavaString {
	inner: Vec<u8>,
}

impl JavaString {
	#[must_use]
	pub const fn new() -> Self {
		Self { inner: Vec::new() }
	}

	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			inner: Vec::with_capacity(capacity),
		}
	}

	pub fn from_full_utf8(v: Vec<u8>) -> Result<Self, FromUtf8Error> {
		match std::str::from_utf8(&v) {
			Ok(..) => Ok(Self { inner: v }),
			Err(e) => Err(FromUtf8Error {
				bytes: v,
				error: e.into(),
			}),
		}
	}

	pub fn from_semi_utf8(v: Vec<u8>) -> Result<Self, FromUtf8Error> {
		match run_utf8_semi_validation(&v) {
			Ok(..) => Ok(Self { inner: v }),
			Err(e) => Err(FromUtf8Error { bytes: v, error: e }),
		}
	}

	#[must_use]
	pub fn from_semi_utf8_lossy(v: &[u8]) -> Cow<'_, JavaStr> {
		const REPLACEMENT: &str = "\u{FFFD}";

		match run_utf8_semi_validation(v) {
			Ok(()) => unsafe { Cow::Borrowed(JavaStr::from_semi_utf8_unchecked(v)) },
			Err(error) => {
				let mut result = unsafe {
					Self::from_semi_utf8_unchecked(v.get_unchecked(..error.valid_up_to).to_owned())
				};
				result.push_str(REPLACEMENT);
				let mut index = error.valid_up_to + error.error_len().unwrap_or(1);
				loop {
					match run_utf8_semi_validation(&v[index..]) {
						Ok(()) => {
							unsafe {
								result
									.push_java_str(JavaStr::from_semi_utf8_unchecked(&v[index..]));
							}
							return Cow::Owned(result);
						}
						Err(error) => {
							unsafe {
								result.push_java_str(JavaStr::from_semi_utf8_unchecked(
									v.get_unchecked(index..index + error.valid_up_to),
								));
							}
							result.push_str(REPLACEMENT);
							index += error.valid_up_to + error.error_len().unwrap_or(1);
						}
					}
				}
			}
		}
	}

	#[must_use]
	pub const unsafe fn from_semi_utf8_unchecked(bytes: Vec<u8>) -> Self {
		Self { inner: bytes }
	}

	#[must_use]
	pub fn into_bytes(self) -> Vec<u8> {
		self.inner
	}

	#[allow(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn as_java_str(&self) -> &JavaStr {
		unsafe { JavaStr::from_semi_utf8_unchecked(&self.inner) }
	}

	pub fn as_mut_java_str(&mut self) -> &mut JavaStr {
		unsafe { JavaStr::from_semi_utf8_unchecked_mut(&mut self.inner) }
	}

	pub fn into_string(self) -> Result<String, Utf8Error> {
		run_utf8_full_validation_from_semi(self.as_bytes())
			.map(|()| unsafe { self.into_string_unchecked() })
	}

	#[must_use]
	pub unsafe fn into_string_unchecked(self) -> String {
		unsafe { String::from_utf8_unchecked(self.inner) }
	}

	pub fn push_java_str(&mut self, string: &JavaStr) {
		self.inner.extend_from_slice(string.as_bytes());
	}

	pub fn push_str(&mut self, string: &str) {
		self.inner.extend_from_slice(string.as_bytes());
	}

	#[must_use]
	pub fn capacity(&self) -> usize {
		self.inner.capacity()
	}

	pub fn reserve(&mut self, additional: usize) {
		self.inner.reserve(additional);
	}

	pub fn reserve_exact(&mut self, additional: usize) {
		self.inner.reserve_exact(additional);
	}

	pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.inner.try_reserve(additional)
	}

	pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.inner.try_reserve_exact(additional)
	}

	pub fn shrink_to_fit(&mut self) {
		self.inner.shrink_to_fit();
	}

	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.inner.shrink_to(min_capacity);
	}

	pub fn push(&mut self, ch: char) {
		match ch.len_utf8() {
			1 => self.inner.push(ch as u8),
			_ => self
				.inner
				.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
		}
	}

	pub fn push_java(&mut self, ch: JavaCodePoint) {
		match ch.len_utf8() {
			1 => self.inner.push(ch.as_u32() as u8),
			_ => self
				.inner
				.extend_from_slice(ch.encode_semi_utf8(&mut [0; 4])),
		}
	}

	#[allow(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.inner
	}

	pub fn truncate(&mut self, new_len: usize) {
		if new_len <= self.len() {
			assert!(self.is_char_boundary(new_len));
			self.inner.truncate(new_len);
		}
	}

	pub fn pop(&mut self) -> Option<JavaCodePoint> {
		let ch = self.chars().next_back()?;
		let new_len = self.len() - ch.len_utf8();
		unsafe {
			self.inner.set_len(new_len);
		}
		Some(ch)
	}

	pub fn remove(&mut self, idx: usize) -> JavaCodePoint {
		let Some(ch) = self[idx..].chars().next() else {
			panic!("cannot remove a char from the end of a string");
		};

		let next = idx + ch.len_utf8();
		let len = self.len();
		unsafe {
			ptr::copy(
				self.inner.as_ptr().add(next),
				self.inner.as_mut_ptr().add(idx),
				len - next,
			);
			self.inner.set_len(len - (next - idx));
		}

		ch
	}

	pub fn retain(&mut self, mut f: impl FnMut(JavaCodePoint) -> bool) {
		struct SetLenOnDrop<'a> {
			s: &'a mut JavaString,
			idx: usize,
			del_bytes: usize,
		}

		impl Drop for SetLenOnDrop<'_> {
			fn drop(&mut self) {
				let new_len = self.idx - self.del_bytes;
				debug_assert!(new_len <= self.s.len());
				unsafe {
					self.s.inner.set_len(new_len);
				}
			}
		}

		let len = self.len();
		let mut guard = SetLenOnDrop {
			s: self,
			idx: 0,
			del_bytes: 0,
		};

		while guard.idx < len {
			let ch = unsafe {
				guard
					.s
					.get_unchecked(guard.idx..len)
					.chars()
					.next()
					.unwrap_unchecked()
			};
			let ch_len = ch.len_utf8();

			if !f(ch) {
				guard.del_bytes += ch_len;
			} else if guard.del_bytes > 0 {
				ch.encode_semi_utf8(unsafe {
					slice::from_raw_parts_mut(
						guard.s.as_mut_ptr().add(guard.idx - guard.del_bytes),
						ch.len_utf8(),
					)
				});
			}

			guard.idx += ch_len;
		}

		drop(guard);
	}

	pub fn insert(&mut self, idx: usize, ch: char) {
		assert!(self.is_char_boundary(idx));
		let mut bits = [0; 4];
		let bits = ch.encode_utf8(&mut bits).as_bytes();

		unsafe {
			self.insert_bytes(idx, bits);
		}
	}

	pub fn insert_java(&mut self, idx: usize, ch: JavaCodePoint) {
		assert!(self.is_char_boundary(idx));
		let mut bits = [0; 4];
		let bits = ch.encode_semi_utf8(&mut bits);

		unsafe {
			self.insert_bytes(idx, bits);
		}
	}

	unsafe fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) {
		let len = self.len();
		let amt = bytes.len();
		self.reserve(amt);

		unsafe {
			ptr::copy(
				self.inner.as_ptr().add(idx),
				self.inner.as_mut_ptr().add(idx + amt),
				len - idx,
			);
			ptr::copy_nonoverlapping(bytes.as_ptr(), self.inner.as_mut_ptr().add(idx), amt);
			self.inner.set_len(len + amt);
		}
	}

	pub fn insert_str(&mut self, idx: usize, string: &str) {
		assert!(self.is_char_boundary(idx));

		unsafe {
			self.insert_bytes(idx, string.as_bytes());
		}
	}

	pub fn insert_java_str(&mut self, idx: usize, string: &JavaStr) {
		assert!(self.is_char_boundary(idx));

		unsafe {
			self.insert_bytes(idx, string.as_bytes());
		}
	}

	pub const fn as_mut_vec(&mut self) -> &mut Vec<u8> {
		&mut self.inner
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.inner.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}

	#[must_use]
	pub fn split_off(&mut self, at: usize) -> Self {
		assert!(self.is_char_boundary(at));
		let other = self.inner.split_off(at);
		unsafe { Self::from_semi_utf8_unchecked(other) }
	}

	pub fn clear(&mut self) {
		self.inner.clear();
	}

	pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Drain<'_> {
		let Range { start, end } = to_range_checked(range, ..self.len());
		assert!(self.is_char_boundary(start));
		assert!(self.is_char_boundary(end));

		let self_ptr = self as *mut _;

		let chars_iter = unsafe { self.get_unchecked(start..end) }.chars();

		Drain {
			start,
			end,
			iter: chars_iter,
			string: self_ptr,
		}
	}

	pub fn replace_range(&mut self, range: impl RangeBounds<usize>, replace_with: &str) {
		self.replace_range_java(range, JavaStr::from_str(replace_with));
	}

	pub fn replace_range_java(&mut self, range: impl RangeBounds<usize>, replace_with: &JavaStr) {
		let start = range.start_bound();
		match start {
			Bound::Included(&n) => assert!(self.is_char_boundary(n)),
			Bound::Excluded(&n) => assert!(self.is_char_boundary(n + 1)),
			Bound::Unbounded => {}
		}

		let end = range.end_bound();
		match end {
			Bound::Included(&n) => assert!(self.is_char_boundary(n + 1)),
			Bound::Excluded(&n) => assert!(self.is_char_boundary(n)),
			Bound::Unbounded => {}
		}

		self.as_mut_vec().splice((start, end), replace_with.bytes());
	}

	#[must_use]
	pub fn into_boxed_str(self) -> Box<JavaStr> {
		let slice = self.inner.into_boxed_slice();
		unsafe { JavaStr::from_boxed_semi_utf8_unchecked(slice) }
	}

	#[must_use]
	pub fn leak<'a>(self) -> &'a mut JavaStr {
		let slice = self.inner.leak();
		unsafe { JavaStr::from_semi_utf8_unchecked_mut(slice) }
	}
}

impl Add<&str> for JavaString {
	type Output = Self;

	fn add(mut self, rhs: &str) -> Self::Output {
		self.push_str(rhs);
		self
	}
}

impl Add<&JavaStr> for JavaString {
	type Output = Self;

	fn add(mut self, rhs: &JavaStr) -> Self::Output {
		self.push_java_str(rhs);
		self
	}
}

impl AddAssign<&str> for JavaString {
	fn add_assign(&mut self, rhs: &str) {
		self.push_str(rhs);
	}
}

impl AddAssign<&JavaStr> for JavaString {
	fn add_assign(&mut self, rhs: &JavaStr) {
		self.push_java_str(rhs);
	}
}

impl AsMut<JavaStr> for JavaString {
	fn as_mut(&mut self) -> &mut JavaStr {
		self.as_mut_java_str()
	}
}

impl AsRef<[u8]> for JavaString {
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl AsRef<JavaStr> for JavaString {
	fn as_ref(&self) -> &JavaStr {
		self.as_java_str()
	}
}

impl Borrow<JavaStr> for JavaString {
	fn borrow(&self) -> &JavaStr {
		self.as_java_str()
	}
}

impl BorrowMut<JavaStr> for JavaString {
	fn borrow_mut(&mut self) -> &mut JavaStr {
		self.as_mut_java_str()
	}
}

impl Clone for JavaString {
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}

	fn clone_from(&mut self, source: &Self) {
		self.inner.clone_from(&source.inner);
	}
}

impl Debug for JavaString {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&**self, f)
	}
}

impl Deref for JavaString {
	type Target = JavaStr;

	fn deref(&self) -> &Self::Target {
		self.as_java_str()
	}
}

impl DerefMut for JavaString {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_java_str()
	}
}

impl Display for JavaString {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&**self, f)
	}
}

impl Extend<char> for JavaString {
	fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
		let iterator = iter.into_iter();
		let (lower_bound, _) = iterator.size_hint();
		self.reserve(lower_bound);
		iterator.for_each(move |c| self.push(c));
	}
}

impl Extend<JavaCodePoint> for JavaString {
	fn extend<T: IntoIterator<Item = JavaCodePoint>>(&mut self, iter: T) {
		let iterator = iter.into_iter();
		let (lower_bound, _) = iterator.size_hint();
		self.reserve(lower_bound);
		iterator.for_each(move |c| self.push_java(c));
	}
}

impl Extend<String> for JavaString {
	fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_str(&s));
	}
}

impl Extend<Self> for JavaString {
	fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_java_str(&s));
	}
}

impl<'a> Extend<&'a char> for JavaString {
	fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
		self.extend(iter.into_iter().copied());
	}
}

impl<'a> Extend<&'a JavaCodePoint> for JavaString {
	fn extend<T: IntoIterator<Item = &'a JavaCodePoint>>(&mut self, iter: T) {
		self.extend(iter.into_iter().copied());
	}
}

impl<'a> Extend<&'a str> for JavaString {
	fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |n| self.push_str(n));
	}
}

impl<'a> Extend<&'a JavaStr> for JavaString {
	fn extend<T: IntoIterator<Item = &'a JavaStr>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_java_str(s));
	}
}

impl Extend<Box<str>> for JavaString {
	fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_str(&s));
	}
}

impl Extend<Box<JavaStr>> for JavaString {
	fn extend<T: IntoIterator<Item = Box<JavaStr>>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_java_str(&s));
	}
}

impl<'a> Extend<Cow<'a, str>> for JavaString {
	fn extend<T: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_str(&s));
	}
}

impl<'a> Extend<Cow<'a, JavaStr>> for JavaString {
	fn extend<T: IntoIterator<Item = Cow<'a, JavaStr>>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |s| self.push_java_str(&s));
	}
}

impl From<String> for JavaString {
	fn from(value: String) -> Self {
		unsafe { Self::from_semi_utf8_unchecked(value.into_bytes()) }
	}
}

impl From<&String> for JavaString {
	fn from(value: &String) -> Self {
		Self::from(value.clone())
	}
}

impl From<&Self> for JavaString {
	fn from(value: &Self) -> Self {
		value.clone()
	}
}

impl From<&mut str> for JavaString {
	fn from(value: &mut str) -> Self {
		Self::from(&*value)
	}
}

impl From<&str> for JavaString {
	fn from(value: &str) -> Self {
		Self::from(value.to_owned())
	}
}

impl From<&mut JavaStr> for JavaString {
	fn from(value: &mut JavaStr) -> Self {
		Self::from(&*value)
	}
}

impl From<&JavaStr> for JavaString {
	fn from(value: &JavaStr) -> Self {
		value.to_owned()
	}
}

impl From<Box<str>> for JavaString {
	fn from(value: Box<str>) -> Self {
		Self::from(value.into_string())
	}
}

impl From<Box<JavaStr>> for JavaString {
	fn from(value: Box<JavaStr>) -> Self {
		value.into_string()
	}
}

impl<'a> From<Cow<'a, str>> for JavaString {
	fn from(value: Cow<'a, str>) -> Self {
		Self::from(value.into_owned())
	}
}

impl From<JavaString> for Arc<JavaStr> {
	fn from(value: JavaString) -> Self {
		Self::from(&value[..])
	}
}

impl From<JavaString> for Cow<'_, JavaStr> {
	fn from(value: JavaString) -> Self {
		Self::Owned(value)
	}
}

impl From<JavaString> for Rc<JavaStr> {
	fn from(value: JavaString) -> Self {
		Self::from(&value[..])
	}
}

impl From<JavaString> for Vec<u8> {
	fn from(value: JavaString) -> Self {
		value.into_bytes()
	}
}

impl From<char> for JavaString {
	fn from(value: char) -> Self {
		Self::from(value.encode_utf8(&mut [0; 4]))
	}
}

impl From<JavaCodePoint> for JavaString {
	fn from(value: JavaCodePoint) -> Self {
		unsafe { Self::from_semi_utf8_unchecked(value.encode_semi_utf8(&mut [0; 4]).to_owned()) }
	}
}

impl<A> FromIterator<A> for JavaString
where
	Self: Extend<A>,
{
	fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
		let mut buf = Self::new();
		buf.extend(iter);
		buf
	}
}

impl FromStr for JavaString {
	type Err = Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(s))
	}
}

impl Hash for JavaString {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(**self).hash(state);
	}
}

impl Index<Range<usize>> for JavaString {
	type Output = JavaStr;

	fn index(&self, index: Range<usize>) -> &Self::Output {
		&self[..][index]
	}
}

impl Index<RangeFrom<usize>> for JavaString {
	type Output = JavaStr;

	fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
		&self[..][index]
	}
}

impl Index<RangeFull> for JavaString {
	type Output = JavaStr;

	fn index(&self, _: RangeFull) -> &Self::Output {
		self.as_java_str()
	}
}

impl Index<RangeInclusive<usize>> for JavaString {
	type Output = JavaStr;

	fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
		&self[..][index]
	}
}

impl Index<RangeTo<usize>> for JavaString {
	type Output = JavaStr;

	fn index(&self, index: RangeTo<usize>) -> &Self::Output {
		&self[..][index]
	}
}

impl Index<RangeToInclusive<usize>> for JavaString {
	type Output = JavaStr;

	fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
		&self[..][index]
	}
}

impl IndexMut<Range<usize>> for JavaString {
	fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
		&mut self[..][index]
	}
}

impl IndexMut<RangeFrom<usize>> for JavaString {
	fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
		&mut self[..][index]
	}
}

impl IndexMut<RangeFull> for JavaString {
	fn index_mut(&mut self, _: RangeFull) -> &mut Self::Output {
		self.as_mut()
	}
}

impl IndexMut<RangeInclusive<usize>> for JavaString {
	fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
		&mut self[..][index]
	}
}

impl IndexMut<RangeTo<usize>> for JavaString {
	fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
		&mut self[..][index]
	}
}

impl IndexMut<RangeToInclusive<usize>> for JavaString {
	fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut Self::Output {
		&mut self[..][index]
	}
}

impl PartialEq<str> for JavaString {
	fn eq(&self, other: &str) -> bool {
		self[..] == other
	}
}

impl PartialEq<JavaString> for str {
	fn eq(&self, other: &JavaString) -> bool {
		self == other[..]
	}
}

impl<'a> PartialEq<&'a str> for JavaString {
	fn eq(&self, other: &&'a str) -> bool {
		self == *other
	}
}

impl PartialEq<JavaString> for &str {
	fn eq(&self, other: &JavaString) -> bool {
		*self == other
	}
}

impl PartialEq<String> for JavaString {
	fn eq(&self, other: &String) -> bool {
		&self[..] == other
	}
}

impl PartialEq<JavaString> for String {
	fn eq(&self, other: &JavaString) -> bool {
		self == &other[..]
	}
}

impl PartialEq<JavaStr> for JavaString {
	fn eq(&self, other: &JavaStr) -> bool {
		self[..] == other
	}
}

impl<'a> PartialEq<&'a JavaStr> for JavaString {
	fn eq(&self, other: &&'a JavaStr) -> bool {
		self == *other
	}
}

impl<'a> PartialEq<Cow<'a, str>> for JavaString {
	fn eq(&self, other: &Cow<'a, str>) -> bool {
		&self[..] == other
	}
}

impl PartialEq<JavaString> for Cow<'_, str> {
	fn eq(&self, other: &JavaString) -> bool {
		self == &other[..]
	}
}

impl<'a> PartialEq<Cow<'a, JavaStr>> for JavaString {
	fn eq(&self, other: &Cow<'a, JavaStr>) -> bool {
		&self[..] == other
	}
}

impl PartialEq<JavaString> for Cow<'_, JavaStr> {
	fn eq(&self, other: &JavaString) -> bool {
		self == &other[..]
	}
}

impl Write for JavaString {
	fn write_str(&mut self, s: &str) -> FmtResult {
		self.push_str(s);
		Ok(())
	}

	fn write_char(&mut self, c: char) -> FmtResult {
		self.push(c);
		Ok(())
	}
}

pub struct Drain<'a> {
	string: *mut JavaString,
	start: usize,
	end: usize,
	iter: Chars<'a>,
}

impl Drain<'_> {
	#[must_use]
	pub fn as_str(&self) -> &JavaStr {
		self.iter.as_str()
	}
}

impl AsRef<JavaStr> for Drain<'_> {
	fn as_ref(&self) -> &JavaStr {
		self.as_str()
	}
}

impl AsRef<[u8]> for Drain<'_> {
	fn as_ref(&self) -> &[u8] {
		self.as_str().as_bytes()
	}
}

impl Debug for Drain<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("Drain").field(&self.as_str()).finish()
	}
}

impl DoubleEndedIterator for Drain<'_> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.iter.next_back()
	}
}

impl Drop for Drain<'_> {
	fn drop(&mut self) {
		unsafe {
			let self_vec = (*self.string).as_mut_vec();
			if self.start <= self.end && self.end <= self_vec.len() {
				self_vec.drain(self.start..self.end);
			}
		}
	}
}

impl FusedIterator for Drain<'_> {}

impl Iterator for Drain<'_> {
	type Item = JavaCodePoint;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}

	fn last(mut self) -> Option<JavaCodePoint> {
		self.next_back()
	}
}

unsafe impl Send for Drain<'_> {}
unsafe impl Sync for Drain<'_> {}
