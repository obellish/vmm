#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

mod sealed;

use core::{
	marker::PhantomData,
	ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

#[repr(transparent)]
pub struct Unbounded<T: ?Sized>(PhantomData<T>);

#[repr(transparent)]
pub struct Included<T: ?Sized>(PhantomData<T>);

#[repr(transparent)]
pub struct Excluded<T: ?Sized>(PhantomData<T>);

pub struct Span<T, From, To>
where
	From: ?Sized + SpanBound<T>,
	To: ?Sized + SpanBound<T>,
{
	start: Bound<T>,
	end: Bound<T>,
	marker_from: PhantomData<From>,
	marker_to: PhantomData<To>,
}

impl<T, From, To> Span<T, From, To>
where
	From: ?Sized + SpanBound<T>,
	To: ?Sized + SpanBound<T>,
{
	pub fn new(start: T, end: T) -> Self {
		Self {
			start: From::into_bound(start),
			end: To::into_bound(end),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<Range<T>> for Spanned<T> {
	fn from(value: Range<T>) -> Self {
		Self::new(value.start, value.end)
	}
}

impl<T> From<RangeFull> for SpannedFull<T> {
	fn from(_: RangeFull) -> Self {
		Self {
			start: Bound::Unbounded,
			end: Bound::Unbounded,
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<RangeFrom<T>> for SpannedFrom<T> {
	fn from(value: RangeFrom<T>) -> Self {
		Self {
			start: Included::<T>::into_bound(value.start),
			end: Bound::Unbounded,
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<RangeTo<T>> for SpannedTo<T> {
	fn from(value: RangeTo<T>) -> Self {
		Self {
			start: Bound::Unbounded,
			end: Excluded::<T>::into_bound(value.end),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

/// Annoying bound needed bc [`RangeInclusive`] doesn't expose `start` and `end`.
impl<T: Clone> From<RangeInclusive<T>> for SpannedInclusive<T> {
	fn from(value: RangeInclusive<T>) -> Self {
		Self {
			start: Bound::Included(value.start().clone()),
			end: Bound::Included(value.end().clone()),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T> From<RangeToInclusive<T>> for SpannedToInclusive<T> {
	fn from(value: RangeToInclusive<T>) -> Self {
		Self {
			start: Bound::Unbounded,
			end: Included::into_bound(value.end),
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

unsafe impl<T: Send, From, To> Send for Span<T, From, To>
where
	From: ?Sized + SpanBound<T>,
	To: ?Sized + SpanBound<T>,
{
}

unsafe impl<T: Sync, From, To> Sync for Span<T, From, To>
where
	From: ?Sized + SpanBound<T>,
	To: ?Sized + SpanBound<T>,
{
}

pub trait SpanBound<T>: self::sealed::Sealed {
	fn into_bound(value: T) -> Bound<T>;
}

impl<T> SpanBound<T> for Unbounded<T> {
	fn into_bound(_: T) -> Bound<T> {
		Bound::Unbounded
	}
}

impl<T> SpanBound<T> for Included<T> {
	fn into_bound(value: T) -> Bound<T> {
		Bound::Included(value)
	}
}

impl<T> SpanBound<T> for Excluded<T> {
	fn into_bound(value: T) -> Bound<T> {
		Bound::Excluded(value)
	}
}

pub type SpannedFull<T> = Span<T, Unbounded<T>, Unbounded<T>>;

pub type Spanned<T> = Span<T, Included<T>, Excluded<T>>;

pub type SpannedFrom<T> = Span<T, Included<T>, Unbounded<T>>;

pub type SpannedTo<T> = Span<T, Unbounded<T>, Excluded<T>>;

pub type SpannedInclusive<T> = Span<T, Included<T>, Included<T>>;

pub type SpannedToInclusive<T> = Span<T, Unbounded<T>, Included<T>>;
