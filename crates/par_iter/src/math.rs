use std::ops::{Bound, Range, RangeBounds};

use vmm_span::{Span, Spanned};

pub(super) fn div_round_up(n: usize, divisor: usize) -> usize {
	debug_assert_ne!(divisor, 0, "division by zero");
	if matches!(n, 0) {
		0
	} else {
		(n - 1) / divisor + 1
	}
}

pub(super) fn simplify_range(range: impl RangeBounds<usize>, len: usize) -> Spanned<usize> {
	let start = match range.start_bound() {
		Bound::Unbounded => 0,
		Bound::Included(&i) if i <= len => i,
		Bound::Excluded(&i) if i < len => i + 1,
		bound => panic!("range start {bound:?} should be <= length {len}"),
	};

	let end = match range.end_bound() {
		Bound::Unbounded => len,
		Bound::Excluded(&i) if i <= len => i,
		Bound::Included(&i) if i < len => i + 1,
		bound => panic!("range end {bound:?} should be <= length {len}"),
	};

	assert!(
		(start <= end),
		"range start {:?} should be <= range end {:?}",
		range.start_bound(),
		range.end_bound()
	);

	Span::from(start..end)
}

#[cfg(test)]
mod tests {
	use super::div_round_up;

	#[test]
	fn div_round_up_works() {
		assert_eq!(div_round_up(0, 5), 0);
		assert_eq!(div_round_up(5, 5), 1);
		assert_eq!(div_round_up(1, 5), 1);
		assert_eq!(div_round_up(3, 2), 2);
		assert_eq!(div_round_up(usize::MAX, 2), usize::MAX / 2 + 1);
	}
}
