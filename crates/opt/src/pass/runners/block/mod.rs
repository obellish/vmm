mod r#dyn;
mod if_nz;

use std::ops::RangeInclusive;

pub use self::{r#dyn::*, if_nz::*};

#[derive(Debug, Clone)]
enum BlockLength {
	Single(usize),
	Range(RangeInclusive<usize>),
	LowerBound(usize),
	Unknown,
}

impl BlockLength {
	const fn new(size_hint: (usize, Option<usize>)) -> Self {
		match size_hint {
			(0, None) => Self::Unknown,
			(value, None) => Self::LowerBound(value),
			(lower, Some(upper)) if lower == upper => Self::Single(lower),
			(lower, Some(upper)) => Self::Range(lower..=upper),
		}
	}
}
