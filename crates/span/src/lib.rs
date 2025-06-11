#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

mod iter;
mod sealed;
mod serde;

use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	marker::PhantomData,
	ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

pub use self::iter::*;

#[repr(transparent)]
pub struct Unbounded<T: ?Sized>(PhantomData<T>);

#[repr(transparent)]
pub struct Included<T: ?Sized>(PhantomData<T>);

#[repr(transparent)]
pub struct Excluded<T: ?Sized>(PhantomData<T>);

pub struct Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	start: Option<T>,
	end: Option<T>,
	marker_from: PhantomData<From>,
	marker_to: PhantomData<To>,
}

impl<T, From, To> Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	pub const fn new(start: T, end: T) -> Self {
		Self {
			start: Some(start),
			end: Some(end),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}

	pub fn start_bound(&self) -> Bound<&T> {
		match &self.start {
			None => Bound::Unbounded,
			Some(value) => From::as_ref_bound(value),
		}
	}

	pub fn end_bound(&self) -> Bound<&T> {
		match &self.end {
			None => Bound::Unbounded,
			Some(value) => To::as_ref_bound(value),
		}
	}

	pub fn contains<U>(&self, item: &U) -> bool
	where
		T: PartialOrd<U>,
		U: ?Sized + PartialOrd<T>,
	{
		(match self.start_bound() {
			Bound::Included(start) => start <= item,
			Bound::Excluded(start) => start < item,
			Bound::Unbounded => true,
		}) && (match self.end_bound() {
			Bound::Included(end) => item <= end,
			Bound::Excluded(end) => item < end,
			Bound::Unbounded => true,
		})
	}

	pub fn is_empty(&self) -> bool
	where
		T: PartialOrd,
	{
		!match (self.start_bound(), self.end_bound()) {
			(Bound::Unbounded, _) | (_, Bound::Unbounded) => true,
			(Bound::Included(start) | Bound::Excluded(start), Bound::Excluded(end))
			| (Bound::Excluded(start), Bound::Included(end)) => start < end,
			(Bound::Included(start), Bound::Included(end)) => start <= end,
		}
	}
}

impl<T: Clone, From, To> Clone for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	fn clone(&self) -> Self {
		Self {
			start: self.start.clone(),
			end: self.end.clone(),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T: Copy, From, To> Copy for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
}

impl<T: Debug, From, To> Debug for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Span")
			.field("start", &self.start)
			.field("end", &self.end)
			.finish()
	}
}

impl<T: Eq, From, To> Eq for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
}

impl<T> From<Range<T>> for Spanned<T> {
	fn from(value: Range<T>) -> Self {
		Self::new(value.start, value.end)
	}
}

impl<T> From<RangeFull> for SpannedFull<T> {
	fn from(_: RangeFull) -> Self {
		Self {
			start: None,
			end: None,
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<RangeFrom<T>> for SpannedFrom<T> {
	fn from(value: RangeFrom<T>) -> Self {
		Self {
			start: Some(value.start),
			end: None,
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<RangeTo<T>> for SpannedTo<T> {
	fn from(value: RangeTo<T>) -> Self {
		Self {
			start: None,
			end: Some(value.end),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

/// Annoying bound needed bc [`RangeInclusive`] doesn't expose `start` and `end`.
impl<T: Clone> From<RangeInclusive<T>> for SpannedInclusive<T> {
	fn from(value: RangeInclusive<T>) -> Self {
		Self {
			start: Some(value.start().clone()),
			end: Some(value.end().clone()),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<RangeToInclusive<T>> for SpannedToInclusive<T> {
	fn from(value: RangeToInclusive<T>) -> Self {
		Self {
			start: None,
			end: Some(value.end),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T: Walk> IntoIterator for Spanned<T> {
	type IntoIter = SpannedIter<T>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		SpanIter::new(self)
	}
}

impl<T: Walk> IntoIterator for SpannedFrom<T> {
	type IntoIter = SpannedFromIter<T>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		SpanIter::new(self)
	}
}

impl<T: Walk> IntoIterator for SpannedInclusive<T> {
	type IntoIter = SpannedInclusiveIter<T>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		SpanIter::new(self)
	}
}

impl<T: PartialEq, From, To> PartialEq for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.start, &other.start) && PartialEq::eq(&self.end, &other.end)
	}
}

impl<T: PartialEq, From, To> PartialEq<SpanIter<T, From, To>> for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	fn eq(&self, other: &SpanIter<T, From, To>) -> bool {
		PartialEq::eq(self, &other.span)
	}
}

unsafe impl<T: Send, From, To> Send for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
}

unsafe impl<T: Sync, From, To> Sync for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
}

pub trait SpanBound<T>: self::sealed::Sealed {
	fn into_bound(item: T) -> Bound<T>;

	fn as_ref_bound(item: &T) -> Bound<&T>;
}

impl<T> SpanBound<T> for Unbounded<T> {
	fn into_bound(_: T) -> Bound<T> {
		Bound::Unbounded
	}

	fn as_ref_bound(_: &T) -> Bound<&T> {
		Bound::Unbounded
	}
}

impl<T> SpanBound<T> for Included<T> {
	fn into_bound(item: T) -> Bound<T> {
		Bound::Included(item)
	}

	fn as_ref_bound(item: &T) -> Bound<&T> {
		Bound::Included(item)
	}
}

impl<T> SpanBound<T> for Excluded<T> {
	fn into_bound(item: T) -> Bound<T> {
		Bound::Excluded(item)
	}

	fn as_ref_bound(item: &T) -> Bound<&T> {
		Bound::Excluded(item)
	}
}

pub trait SpanStartBound<T>: SpanBound<T> {}

impl<T> SpanStartBound<T> for Unbounded<T> {}

impl<T> SpanStartBound<T> for Included<T> {}

pub type SpannedFull<T> = Span<T, Unbounded<T>, Unbounded<T>>;

pub type Spanned<T> = Span<T, Included<T>, Excluded<T>>;

pub type SpannedFrom<T> = Span<T, Included<T>, Unbounded<T>>;

pub type SpannedTo<T> = Span<T, Unbounded<T>, Excluded<T>>;

pub type SpannedInclusive<T> = Span<T, Included<T>, Included<T>>;

pub type SpannedToInclusive<T> = Span<T, Unbounded<T>, Included<T>>;
