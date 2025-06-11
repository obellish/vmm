use core::{cmp, num::NonZeroU8};

use serde::{Deserialize, Serialize};
use vmm_span::SpannedInclusive;

use super::{IsZeroingCell, Offset, PtrMovement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpanInstruction {
	pub ty: SpanInstructionType,
	pub start: Offset,
	pub end: Offset,
}

impl SpanInstruction {
	#[must_use]
	pub fn inc_span(value: i8, start: impl Into<Offset>, end: impl Into<Offset>) -> Self {
		Self::from_range(SpanInstructionType::Inc { value }, start.into(), end.into())
	}

	#[must_use]
	pub fn set_span(value: u8, start: impl Into<Offset>, end: impl Into<Offset>) -> Self {
		Self::from_range(
			SpanInstructionType::Set {
				value: NonZeroU8::new(value),
			},
			start.into(),
			end.into(),
		)
	}

	#[must_use]
	pub fn clear_range(start: impl Into<Offset>, end: impl Into<Offset>) -> Self {
		Self::set_span(0, start, end)
	}

	#[must_use]
	pub const fn is_set(self) -> bool {
		self.ty().is_set()
	}

	#[must_use]
	pub const fn is_inc(self) -> bool {
		self.ty().is_inc()
	}

	#[must_use]
	pub const fn is_clear(self) -> bool {
		self.ty().is_clear()
	}

	#[must_use]
	pub const fn ty(self) -> SpanInstructionType {
		self.ty
	}

	#[must_use]
	pub const fn span(self) -> SpannedInclusive<Offset> {
		SpannedInclusive::new(self.start, self.end)
	}

	#[inline]
	fn from_range(kind: SpanInstructionType, start: Offset, end: Offset) -> Self {
		Self {
			ty: kind,
			start: cmp::min(start, end),
			end: cmp::max(start, end),
		}
	}
}

impl IsZeroingCell for SpanInstruction {
	#[inline]
	fn is_zeroing_cell(&self) -> bool {
		self.is_clear() && self.span().contains(&0)
	}
}

impl PtrMovement for SpanInstruction {
	#[inline]
	fn ptr_movement(&self) -> Option<isize> {
		Some(0)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpanInstructionType {
	Inc { value: i8 },
	Set { value: Option<NonZeroU8> },
}

impl SpanInstructionType {
	#[must_use]
	pub const fn is_inc(self) -> bool {
		matches!(self, Self::Inc { .. })
	}

	#[must_use]
	pub const fn is_set(self) -> bool {
		matches!(self, Self::Set { .. })
	}

	#[must_use]
	pub const fn is_clear(self) -> bool {
		matches!(self, Self::Set { value: None })
	}
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use super::SpanInstruction;

	#[test]
	fn range_matches_expected() {
		let span = SpanInstruction::clear_range(-2, 2);

		let range = span.span().into_iter().collect::<Vec<_>>();

		assert_eq!(range, [-2, -1, 0, 1, 2]);
	}
}
