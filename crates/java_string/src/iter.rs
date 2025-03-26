use std::{
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	iter::{Chain, Copied, Filter, FlatMap, Flatten, FusedIterator, Map},
	option, slice,
};

use super::{
	CharEscapeIter, JavaCodePoint, JavaStr, JavaStrPattern,
	validations::{next_code_point, next_code_point_reverse},
};

macro_rules! delegate {
	(Iterator for $ty:ident $(<$($lt:lifetime),+>)? => $item:ty $(, DoubleEnded = $double_ended:ty)?) => {
        #[allow(unused_imports)]
        impl$(<$($lt),+>)? ::std::iter::Iterator for $ty$(<$($lt),+>)? {
            type Item = $item;

            fn next(&mut self) -> ::std::option::Option<Self::Item> {
                use ::std::iter::Iterator as _;

                self.inner.next()
            }

            fn size_hint(&self) -> (usize, ::std::option::Option<usize>) {
                use ::std::iter::Iterator as _;

                self.inner.size_hint()
            }

            fn count(self) -> usize {
                use ::std::iter::Iterator as _;

                self.inner.count()
            }

            fn last(self) -> ::std::option::Option<Self::Item> {
                use ::std::iter::Iterator as _;

                self.inner.last()
            }

            fn nth(&mut self, n: usize) -> ::std::option::Option<Self::Item> {
                use ::std::iter::Iterator as _;

                self.inner.nth(n)
            }

            fn all<F>(&mut self, f: F) -> bool
            where
                F: ::std::ops::FnMut(Self::Item) -> bool,
            {
                use ::std::iter::Iterator as _;

                self.inner.all(f)
            }

            fn any<F>(&mut self, f: F) -> bool
            where
                F: ::std::ops::FnMut(Self::Item) -> bool,
            {
                use ::std::iter::Iterator as _;

                self.inner.any(f)
            }

            fn find<P>(&mut self, predicate: P) -> ::std::option::Option<Self::Item>
            where
                P: ::std::ops::FnMut(&Self::Item) -> bool,
            {
                use ::std::iter::Iterator as _;

                self.inner.find(predicate)
            }

            fn position<P>(&mut self, predicate: P) -> ::std::option::Option<usize>
            where
                P: ::std::ops::FnMut(Self::Item) -> bool,
            {
                use ::std::iter::Iterator as _;

                self.inner.position(predicate)
            }

            $(

                fn rposition<P>(&mut self, predicate: P) -> ::std::option::Option<usize>
                where
                    P: ::std::ops::FnMut(Self::Item) -> bool,
                {
                    use ::std::iter::Iterator as _;

                    let _test: $double_ended = ();
                    self.inner.rposition(predicate)
            }
            )?
            }
        };
    (DoubleEndedIterator for $ty:ident $(<$($lt:lifetime),+>)?) => {
        #[allow(unused_imports)]
        impl$(<$($lt),+>)? ::std::iter::DoubleEndedIterator for $ty$(<$($lt),+>)? {
            fn next_back(&mut self) -> ::std::option::Option<Self::Item> {
                use ::std::iter::DoubleEndedIterator as _;

                self.inner.next_back()
            }

            fn nth_back(&mut self, n: usize) -> ::std::option::Option<Self::Item> {
                use ::std::iter::DoubleEndedIterator as _;

                self.inner.nth_back(n)
            }

            fn rfind<P>(&mut self, predicate: P) -> ::std::option::Option<Self::Item>
            where
                P: ::std::ops::FnMut(&Self::Item) -> bool,
            {
                use ::std::iter::DoubleEndedIterator as _;

                self.inner.rfind(predicate)
            }
        }
    };
	(ExactSizeIterator for $ty:ident $(<$($lt:lifetime),+>)?) => {
        #[allow(unused_imports)]
        impl$(<$($lt),+>)? ::std::iter::ExactSizeIterator for $ty$(<$($lt),+>)? {
            fn len(&self) -> usize {
                use ::std::iter::ExactSizeIterator as _;

                self.inner.len()
            }
        }
    };
	(FusedIterator for $ty:ident $(<$($lt:lifetime),+>)?) => {
        impl$(<$($lt),+>)? ::std::iter::FusedIterator for $ty$(<$($lt),+>)? {}
    };
	(Iterator, DoubleEndedIterator, ExactSizeIterator, FusedIterator for $ty:ident $(<$($lt:lifetime),+>)? => $item:ty) => {
        delegate!(Iterator for $ty$(<$($lt),+>)? => $item, DoubleEnded = ());
        delegate!(DoubleEndedIterator for $ty$(<$($lt),+>)?);
        delegate!(ExactSizeIterator for $ty$(<$($lt),+>)?);
        delegate!(FusedIterator for $ty$(<$($lt),+>)?);
    };
}

#[must_use]
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Bytes<'a> {
	pub(crate) inner: Copied<slice::Iter<'a, u8>>,
}

delegate!(Iterator, DoubleEndedIterator, ExactSizeIterator, FusedIterator for Bytes<'a> => u8);

#[derive(Debug, Clone)]
#[repr(transparent)]
#[must_use]
pub struct EscapeDebug<'a> {
	pub(crate) inner: Chain<
		Flatten<option::IntoIter<CharEscapeIter>>,
		FlatMap<Chars<'a>, CharEscapeIter, fn(JavaCodePoint) -> CharEscapeIter>,
	>,
}

impl Display for EscapeDebug<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.clone().try_for_each(|c| f.write_char(c))
	}
}

delegate!(Iterator for EscapeDebug<'a> => char);
delegate!(FusedIterator for EscapeDebug<'a>);

#[derive(Debug, Clone)]
#[must_use]
#[repr(transparent)]
pub struct EscapeDefault<'a> {
	pub(crate) inner: FlatMap<Chars<'a>, CharEscapeIter, fn(JavaCodePoint) -> CharEscapeIter>,
}

impl Display for EscapeDefault<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.clone().try_for_each(|c| f.write_char(c))
	}
}

delegate!(Iterator for EscapeDefault<'a> => char);
delegate!(FusedIterator for EscapeDefault<'a>);

#[derive(Debug, Clone)]
#[must_use]
#[repr(transparent)]
pub struct EscapeUnicode<'a> {
	pub(crate) inner: FlatMap<Chars<'a>, CharEscapeIter, fn(JavaCodePoint) -> CharEscapeIter>,
}

impl Display for EscapeUnicode<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.clone().try_for_each(|c| f.write_char(c))
	}
}

delegate!(Iterator for EscapeUnicode<'a> => char);
delegate!(FusedIterator for EscapeUnicode<'a>);

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SplitAsciiWhitespace<'a> {
	pub(crate) inner: Map<
		Filter<slice::Split<'a, u8, fn(&u8) -> bool>, fn(&&[u8]) -> bool>,
		fn(&[u8]) -> &JavaStr,
	>,
}

delegate!(Iterator for SplitAsciiWhitespace<'a> => &'a JavaStr);
delegate!(DoubleEndedIterator for SplitAsciiWhitespace<'a>);
delegate!(FusedIterator for SplitAsciiWhitespace<'a>);

pub struct SplitWhitespace<'a> {
	pub(crate) inner: Filter<Split<'a, fn(JavaCodePoint) -> bool>, fn(&&JavaStr) -> bool>,
}

delegate!(Iterator for SplitWhitespace<'a> => &'a JavaStr);
delegate!(DoubleEndedIterator for SplitWhitespace<'a>);
delegate!(FusedIterator for SplitWhitespace<'a>);

#[derive(Debug, Clone)]
#[must_use]
#[repr(transparent)]
pub struct Lines<'a> {
	pub(crate) inner: Map<SplitInclusive<'a, char>, fn(&JavaStr) -> &JavaStr>,
}

delegate!(Iterator for Lines<'a> => &'a JavaStr);
delegate!(DoubleEndedIterator for Lines<'a>);
delegate!(FusedIterator for Lines<'a>);

#[derive(Clone)]
#[must_use]
#[repr(transparent)]
pub struct Chars<'a> {
	pub(crate) inner: slice::Iter<'a, u8>,
}

impl<'a> Chars<'a> {
	#[must_use]
	pub fn as_str(&self) -> &'a JavaStr {
		unsafe { JavaStr::from_semi_utf8_unchecked(self.inner.as_slice()) }
	}
}

impl Debug for Chars<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Chars(")?;
		f.debug_list().entries(self.clone()).finish()?;
		f.write_char(')')
	}
}

impl DoubleEndedIterator for Chars<'_> {
	fn next_back(&mut self) -> Option<Self::Item> {
		unsafe {
			next_code_point_reverse(&mut self.inner).map(|ch| JavaCodePoint::from_u32_unchecked(ch))
		}
	}
}

impl FusedIterator for Chars<'_> {}

impl Iterator for Chars<'_> {
	type Item = JavaCodePoint;

	fn next(&mut self) -> Option<Self::Item> {
		unsafe { next_code_point(&mut self.inner).map(|ch| JavaCodePoint::from_u32_unchecked(ch)) }
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.inner.len();
		(len.div_ceil(4), Some(len))
	}

	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}
}

#[derive(Debug, Clone)]
#[must_use]
pub struct CharIndices<'a> {
	pub(crate) front_offset: usize,
	pub(crate) inner: Chars<'a>,
}

impl<'a> CharIndices<'a> {
	#[must_use]
	pub fn as_str(&self) -> &'a JavaStr {
		self.inner.as_str()
	}
}

impl DoubleEndedIterator for CharIndices<'_> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next_back().map(|ch| {
			let index = self.front_offset + self.inner.inner.len();
			(index, ch)
		})
	}
}

impl FusedIterator for CharIndices<'_> {}

impl Iterator for CharIndices<'_> {
	type Item = (usize, JavaCodePoint);

	fn next(&mut self) -> Option<Self::Item> {
		let pre_len = self.inner.inner.len();
		let ch = self.inner.next()?;
		let index = self.front_offset;
		let len = self.inner.inner.len();
		self.front_offset += pre_len - len;
		Some((index, ch))
	}

	fn count(self) -> usize {
		self.inner.count()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Matches<'a, P> {
	pub(crate) str: &'a JavaStr,
	pub(crate) pat: P,
}

impl<P: JavaStrPattern> DoubleEndedIterator for Matches<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if let Some((index, len)) = self.pat.rfind_in(self.str) {
			let ret = unsafe { self.str.get_unchecked(index..index + len) };
			self.str = unsafe { self.str.get_unchecked(..index) };
			Some(ret)
		} else {
			self.str = Default::default();
			None
		}
	}
}

impl<'a, P: JavaStrPattern> Iterator for Matches<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((index, len)) = self.pat.find_in(self.str) {
			let ret = unsafe { self.str.get_unchecked(index..index + len) };
			self.str = unsafe { self.str.get_unchecked(index + len..) };
			Some(ret)
		} else {
			self.str = Default::default();
			None
		}
	}
}

#[derive(Debug, Clone)]
#[must_use]
#[repr(transparent)]
pub struct RMatches<'a, P> {
	pub(crate) inner: Matches<'a, P>,
}

impl<P: JavaStrPattern> DoubleEndedIterator for RMatches<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'a, P: JavaStrPattern> Iterator for RMatches<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next_back()
	}
}

#[derive(Debug, Clone)]
#[must_use]
pub struct MatchIndices<'a, P> {
	pub(crate) str: &'a JavaStr,
	pub(crate) start: usize,
	pub(crate) pat: P,
}

impl<P: JavaStrPattern> DoubleEndedIterator for MatchIndices<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if let Some((index, len)) = self.pat.rfind_in(self.str) {
			let ret = unsafe { self.str.get_unchecked(index..index + len) };
			self.str = unsafe { self.str.get_unchecked(..index) };
			Some((self.start + index, ret))
		} else {
			self.str = Default::default();
			None
		}
	}
}

impl<'a, P: JavaStrPattern> Iterator for MatchIndices<'a, P> {
	type Item = (usize, &'a JavaStr);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((index, len)) = self.pat.find_in(self.str) {
			let full_index = self.start + index;
			self.start = full_index + len;

			let ret = unsafe { self.str.get_unchecked(index..index + len) };
			self.str = unsafe { self.str.get_unchecked(index + len..) };
			Some((full_index, ret))
		} else {
			self.start += self.str.len();
			self.str = Default::default();
			None
		}
	}
}

#[derive(Debug, Clone)]
pub struct RMatchIndices<'a, P> {
	pub(crate) inner: MatchIndices<'a, P>,
}

impl<P: JavaStrPattern> DoubleEndedIterator for RMatchIndices<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'a, P: JavaStrPattern> Iterator for RMatchIndices<'a, P> {
	type Item = (usize, &'a JavaStr);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next_back()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Split<'a, P> {
	inner: SplitHelper<'a, P>,
}

impl<'a, P: JavaStrPattern> Split<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, true),
		}
	}
}

impl<P: JavaStrPattern> DoubleEndedIterator for Split<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next_back()
	}
}

impl<P: JavaStrPattern> FusedIterator for Split<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for Split<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RSplit<'a, P> {
	inner: SplitHelper<'a, P>,
}

impl<'a, P: JavaStrPattern> RSplit<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, true),
		}
	}
}

impl<P: JavaStrPattern> DoubleEndedIterator for RSplit<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<P: JavaStrPattern> FusedIterator for RSplit<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for RSplit<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next_back()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SplitTerminator<'a, P> {
	inner: SplitHelper<'a, P>,
}

impl<'a, P: JavaStrPattern> SplitTerminator<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, false),
		}
	}
}

impl<P: JavaStrPattern> DoubleEndedIterator for SplitTerminator<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next_back()
	}
}

impl<P: JavaStrPattern> FusedIterator for SplitTerminator<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for SplitTerminator<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RSplitTerminator<'a, P> {
	inner: SplitHelper<'a, P>,
}

impl<'a, P: JavaStrPattern> RSplitTerminator<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, false),
		}
	}
}

impl<P: JavaStrPattern> DoubleEndedIterator for RSplitTerminator<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<P: JavaStrPattern> FusedIterator for RSplitTerminator<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for RSplitTerminator<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next_back()
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SplitInclusive<'a, P> {
	inner: SplitHelper<'a, P>,
}

impl<'a, P: JavaStrPattern> SplitInclusive<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, false),
		}
	}
}

impl<P: JavaStrPattern> DoubleEndedIterator for SplitInclusive<'_, P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next_back_inclusive()
	}
}

impl<P: JavaStrPattern> FusedIterator for SplitInclusive<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for SplitInclusive<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next_inclusive()
	}
}

#[derive(Debug, Clone)]
pub struct SplitN<'a, P> {
	inner: SplitHelper<'a, P>,
	count: usize,
}

impl<'a, P: JavaStrPattern> SplitN<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P, count: usize) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, true),
			count,
		}
	}
}

impl<P: JavaStrPattern> FusedIterator for SplitN<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for SplitN<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		match self.count {
			0 => None,
			1 => {
				self.count = 0;
				self.inner.get_end()
			}
			_ => {
				self.count -= 1;
				self.inner.next()
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct RSplitN<'a, P> {
	inner: SplitHelper<'a, P>,
	count: usize,
}

impl<'a, P: JavaStrPattern> RSplitN<'a, P> {
	pub(crate) const fn new(haystack: &'a JavaStr, pat: P, count: usize) -> Self {
		Self {
			inner: SplitHelper::new(haystack, pat, true),
			count,
		}
	}
}

impl<P: JavaStrPattern> FusedIterator for RSplitN<'_, P> {}

impl<'a, P: JavaStrPattern> Iterator for RSplitN<'a, P> {
	type Item = &'a JavaStr;

	fn next(&mut self) -> Option<Self::Item> {
		match self.count {
			0 => None,
			1 => {
				self.count = 0;
				self.inner.get_end()
			}
			_ => {
				self.count -= 1;
				self.inner.next_back()
			}
		}
	}
}

#[derive(Debug, Clone)]
struct SplitHelper<'a, P> {
	start: usize,
	end: usize,
	haystack: &'a JavaStr,
	pat: P,
	allow_trailing_empty: bool,
	finished: bool,
	had_empty_matches: bool,
}

impl<'a, P: JavaStrPattern> SplitHelper<'a, P> {
	const fn new(haystack: &'a JavaStr, pat: P, allow_trailing_empty: bool) -> Self {
		Self {
			start: 0,
			end: haystack.len(),
			haystack,
			pat,
			allow_trailing_empty,
			finished: false,
			had_empty_matches: false,
		}
	}

	fn get_end(&mut self) -> Option<&'a JavaStr> {
		if !self.finished {
			self.finished = true;

			if self.allow_trailing_empty || self.end - self.start > 0 {
				let string = unsafe { self.haystack.get_unchecked(self.start..self.end) };
				return Some(string);
			}
		}

		None
	}

	fn next_match(&mut self) -> Option<(usize, usize)> {
		let substr = unsafe { self.haystack.get_unchecked(self.start..) };
		let result = if self.had_empty_matches {
			if substr.is_empty() {
				None
			} else {
				let first_char_len = unsafe { substr.chars().next().unwrap_unchecked().len_utf8() };
				let popped_str = unsafe { substr.get_unchecked(first_char_len..) };

				self.pat
					.find_in(popped_str)
					.map(|(index, len)| (index + first_char_len + self.start, len))
			}
		} else {
			self.pat
				.find_in(substr)
				.map(|(index, len)| (index + self.start, len))
		};

		self.had_empty_matches = result.is_some_and(|(.., len)| matches!(len, 0));

		result
	}

	fn next(&mut self) -> Option<&'a JavaStr> {
		if self.finished {
			return None;
		}

		match self.next_match() {
			Some((index, len)) => unsafe {
				let elt = self.haystack.get_unchecked(self.start..index);
				self.start = index + len;
				Some(elt)
			},
			None => self.get_end(),
		}
	}

	fn next_inclusive(&mut self) -> Option<&'a JavaStr> {
		if self.finished {
			return None;
		}

		match self.next_match() {
			Some((index, len)) => unsafe {
				let elt = self.haystack.get_unchecked(self.start..index + len);
				self.start = index + len;
				Some(elt)
			},
			None => self.get_end(),
		}
	}

	fn next_match_back(&mut self) -> Option<(usize, usize)> {
		let substr = unsafe { self.haystack.get_unchecked(..self.end) };

		let result = if self.had_empty_matches {
			if substr.is_empty() {
				None
			} else {
				let last_char_len =
					unsafe { substr.chars().next_back().unwrap_unchecked().len_utf8() };
				let popped_str = unsafe { substr.get_unchecked(..substr.len() - last_char_len) };

				self.pat.rfind_in(popped_str)
			}
		} else {
			self.pat.rfind_in(substr)
		};

		self.had_empty_matches = result.is_some_and(|(.., len)| matches!(len, 0));

		result
	}

	fn next_back(&mut self) -> Option<&'a JavaStr> {
		if self.finished {
			return None;
		}

		if !self.allow_trailing_empty {
			self.allow_trailing_empty = true;

			match self.next_back() {
				Some(elt) if !elt.is_empty() => return Some(elt),
				_ => {
					if self.finished {
						return None;
					}
				}
			}
		}

		if let Some((index, len)) = self.next_match_back() {
			unsafe {
				let elt = self.haystack.get_unchecked(index + len..self.end);
				self.end = index;
				Some(elt)
			}
		} else {
			unsafe {
				self.finished = true;
				Some(self.haystack.get_unchecked(self.start..self.end))
			}
		}
	}

	fn next_back_inclusive(&mut self) -> Option<&'a JavaStr> {
		if self.finished {
			return None;
		}

		if !self.allow_trailing_empty {
			self.allow_trailing_empty = true;
			match self.next_back_inclusive() {
				Some(elt) if !elt.is_empty() => return Some(elt),
				_ => {
					if self.finished {
						return None;
					}
				}
			}
		}

		if let Some((index, len)) = self.next_match_back() {
			let elt = unsafe { self.haystack.get_unchecked(index + len..self.end) };
			self.end = index + len;
			Some(elt)
		} else {
			self.finished = true;
			Some(unsafe { self.haystack.get_unchecked(self.start..self.end) })
		}
	}
}
