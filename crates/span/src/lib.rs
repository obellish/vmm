#![allow(clippy::fallible_impl_from)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

mod iter;
mod par_iter;
mod sealed;
mod serde;

use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	marker::PhantomData,
	ops::{
		Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
	},
};

use rayon::prelude::*;

pub use self::{iter::*, par_iter::*};

#[repr(transparent)]
pub struct Unbounded<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Debug for Unbounded<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Unbounded")
	}
}

impl<T: ?Sized> Clone for Unbounded<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for Unbounded<T> {}

impl<T: ?Sized> Eq for Unbounded<T> {}

impl<T: ?Sized> PartialEq for Unbounded<T> {
	fn eq(&self, _: &Self) -> bool {
		true
	}
}

unsafe impl<T: ?Sized> Send for Unbounded<T> {}
unsafe impl<T: ?Sized> Sync for Unbounded<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Included<T>(T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Excluded<T>(T);

#[derive(Clone, Copy)]
pub struct Span<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	start: From,
	end: To,
	marker: PhantomData<T>,
}

impl<T, From, To> Span<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	pub fn start_bound(&self) -> Bound<&T> {
		self.start.as_bound()
	}

	pub fn end_bound(&self) -> Bound<&T> {
		self.end.as_bound()
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

impl<T: Debug, From, To> Debug for Span<T, From, To>
where
	From: Debug + SpanStartBound<T>,
	To: Debug + SpanBound<T>,
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
	From: Eq + SpanStartBound<T>,
	To: Eq + SpanBound<T>,
{
}

impl<T> From<Range<T>> for Spanned<T> {
	fn from(value: Range<T>) -> Self {
		Self {
			start: Included(value.start),
			end: Excluded(value.end),
			marker: PhantomData,
		}
	}
}

impl<T> From<RangeFull> for SpannedFull<T> {
	fn from(_: RangeFull) -> Self {
		Self {
			start: Unbounded(PhantomData),
			end: Unbounded(PhantomData),
			marker: PhantomData,
		}
	}
}

impl<T> From<RangeFrom<T>> for SpannedFrom<T> {
	fn from(value: RangeFrom<T>) -> Self {
		Self {
			start: Included(value.start),
			end: Unbounded(PhantomData),
			marker: PhantomData,
		}
	}
}

impl<T> From<RangeTo<T>> for SpannedTo<T> {
	fn from(value: RangeTo<T>) -> Self {
		Self {
			start: Unbounded(PhantomData),
			end: Excluded(value.end),
			marker: PhantomData,
		}
	}
}

/// Annoying bound needed bc [`RangeInclusive`] doesn't expose `start` and `end`.
impl<T: Clone> From<RangeInclusive<T>> for SpannedInclusive<T> {
	fn from(value: RangeInclusive<T>) -> Self {
		Self {
			start: Included(value.start().clone()),
			end: Included(value.end().clone()),
			marker: PhantomData,
		}
	}
}

impl<T> From<RangeToInclusive<T>> for SpannedToInclusive<T> {
	fn from(value: RangeToInclusive<T>) -> Self {
		Self {
			start: Unbounded(PhantomData),
			end: Included(value.end),
			marker: PhantomData,
		}
	}
}

impl<T> From<Spanned<T>> for Range<T> {
	fn from(value: Spanned<T>) -> Self {
		value.start.0..value.end.0
	}
}

impl<T> From<SpannedFull<T>> for RangeFull {
	fn from(_: SpannedFull<T>) -> Self {
		..
	}
}

impl<T> From<SpannedFrom<T>> for RangeFrom<T> {
	fn from(value: SpannedFrom<T>) -> Self {
		value.start.0..
	}
}

impl<T> From<SpannedTo<T>> for RangeTo<T> {
	fn from(value: SpannedTo<T>) -> Self {
		..value.end.0
	}
}

impl<T> From<SpannedInclusive<T>> for RangeInclusive<T> {
	fn from(value: SpannedInclusive<T>) -> Self {
		value.start.0..=value.end.0
	}
}

impl<T> From<SpannedToInclusive<T>> for RangeToInclusive<T> {
	fn from(value: SpannedToInclusive<T>) -> Self {
		..=value.end.0
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

impl<T> IntoParallelIterator for Spanned<T>
where
	T: ParWalk + Send,
{
	type Item = T;
	type Iter = SpanParIter<T>;

	fn into_par_iter(self) -> Self::Iter {
		SpanParIter::new(self.into_iter())
	}
}

impl<T> IntoParallelIterator for SpannedInclusive<T>
where
	RangeInclusive<T>: IntoParallelIterator,
{
	type Item = <RangeInclusive<T> as IntoParallelIterator>::Item;
	type Iter = <RangeInclusive<T> as IntoParallelIterator>::Iter;

	fn into_par_iter(self) -> Self::Iter {
		RangeInclusive::from(self).into_par_iter()
	}
}

impl<T: PartialEq, From, To> PartialEq for Span<T, From, To>
where
	From: PartialEq + SpanStartBound<T>,
	To: PartialEq + SpanBound<T>,
{
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.start, &other.start) && PartialEq::eq(&self.end, &other.end)
	}
}

impl<T: PartialEq, From, To> PartialEq<SpanIter<T, From, To>> for Span<T, From, To>
where
	From: PartialEq + SpanStartBound<T>,
	To: PartialEq + SpanBound<T>,
{
	fn eq(&self, other: &SpanIter<T, From, To>) -> bool {
		PartialEq::eq(self, &other.span)
	}
}

impl<T, From, To> RangeBounds<T> for Span<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	fn start_bound(&self) -> Bound<&T> {
		Self::start_bound(self)
	}

	fn end_bound(&self) -> Bound<&T> {
		Self::end_bound(self)
	}
}

pub trait SpanBound<T>: self::sealed::Sealed {
	fn as_bound(&self) -> Bound<&T>;
}

impl<T> SpanBound<T> for Unbounded<T> {
	fn as_bound(&self) -> Bound<&T> {
		Bound::Unbounded
	}
}

impl<T> SpanBound<T> for Included<T> {
	fn as_bound(&self) -> Bound<&T> {
		Bound::Included(&self.0)
	}
}

impl<T> SpanBound<T> for Excluded<T> {
	fn as_bound(&self) -> Bound<&T> {
		Bound::Excluded(&self.0)
	}
}

pub trait SpanStartBound<T>: SpanBound<T> {}

impl<T> SpanStartBound<T> for Unbounded<T> {}

impl<T> SpanStartBound<T> for Included<T> {}

pub trait SpanBoundValue<T>: SpanBound<T> {
	fn value(&self) -> &T;

	fn value_mut(&mut self) -> &mut T;

	fn into_value(self) -> T;
}

impl<T> SpanBoundValue<T> for Included<T> {
	fn value(&self) -> &T {
		&self.0
	}

	fn value_mut(&mut self) -> &mut T {
		&mut self.0
	}

	fn into_value(self) -> T {
		self.0
	}
}

impl<T> SpanBoundValue<T> for Excluded<T> {
	fn value(&self) -> &T {
		&self.0
	}

	fn value_mut(&mut self) -> &mut T {
		&mut self.0
	}

	fn into_value(self) -> T {
		self.0
	}
}

pub type SpannedFull<T> = Span<T, Unbounded<T>, Unbounded<T>>;

pub type Spanned<T> = Span<T, Included<T>, Excluded<T>>;

pub type SpannedFrom<T> = Span<T, Included<T>, Unbounded<T>>;

pub type SpannedTo<T> = Span<T, Unbounded<T>, Excluded<T>>;

pub type SpannedInclusive<T> = Span<T, Included<T>, Included<T>>;

pub type SpannedToInclusive<T> = Span<T, Unbounded<T>, Included<T>>;
