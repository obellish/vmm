mod iter;
mod serde;

use core::{
	cmp::Ordering,
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Bound, RangeBounds, RangeInclusive},
};

pub use self::iter::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanInclusive<Idx> {
	start: Idx,
	end: Idx,
}

impl<Idx> SpanInclusive<Idx> {
	pub const fn new(start: Idx, end: Idx) -> Self {
		Self { start, end }
	}

	pub const fn start(&self) -> &Idx {
		&self.start
	}

	pub const fn end(&self) -> &Idx {
		&self.end
	}

	pub fn into_inner(self) -> (Idx, Idx) {
		(self.start, self.end)
	}
}

impl<Idx: PartialOrd> SpanInclusive<Idx> {
	pub fn is_empty(&self) -> bool {
		matches!(
			self.start.partial_cmp(&self.end),
			None | Some(Ordering::Greater)
		)
	}

	pub fn contains<U>(&self, item: &U) -> bool
	where
		Idx: PartialOrd<U>,
		U: ?Sized + PartialOrd<Idx>,
	{
		<Self as RangeBounds<Idx>>::contains(self, item)
	}
}

impl<Idx: Debug> Debug for SpanInclusive<Idx> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.start, f)?;
		f.write_str("..=")?;
		Debug::fmt(&self.end, f)
	}
}

impl<Idx> From<RangeInclusive<Idx>> for SpanInclusive<Idx> {
	fn from(value: RangeInclusive<Idx>) -> Self {
		let (start, end) = value.into_inner();
		Self::new(start, end)
	}
}

impl<Idx> From<SpanInclusive<Idx>> for RangeInclusive<Idx> {
	fn from(value: SpanInclusive<Idx>) -> Self {
		let (start, end) = value.into_inner();
		Self::new(start, end)
	}
}

impl<Idx: Step> IntoIterator for SpanInclusive<Idx> {
	type IntoIter = IterSpanInclusive<Idx>;
	type Item = Idx;

	fn into_iter(self) -> Self::IntoIter {
		IterSpanInclusive::new(self.start, self.end)
	}
}

impl<T> RangeBounds<T> for SpanInclusive<T> {
	fn start_bound(&self) -> Bound<&T> {
		Bound::Included(&self.start)
	}

	fn end_bound(&self) -> Bound<&T> {
		Bound::Included(&self.end)
	}
}
