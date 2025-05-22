use core::ops::{
	Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

pub trait IsClosedRange<T> {
	type Range: RangeBounds<T>;

	fn is_range(&self) -> bool;

	fn to_range(&self) -> Option<Self::Range> {
		if self.is_range() {
			Some(unsafe { self.to_range_unchecked() })
		} else {
			None
		}
	}

	unsafe fn to_range_unchecked(&self) -> Self::Range;
}

impl IsClosedRange<usize> for [usize] {
	type Range = RangeInclusive<usize>;

	fn is_range(&self) -> bool {
		if self.is_empty() {
			return false;
		}

		let mut last_value = self[0];

		for i in self.iter().skip(1).copied() {
			if i - 1 != last_value {
				return false;
			}

			last_value = i;
		}

		true
	}

	unsafe fn to_range_unchecked(&self) -> Self::Range {
		let first = unsafe { *self.get_unchecked(0) };
		let last = unsafe { *self.get_unchecked(self.len() - 1) };

		first..=last
	}
}
