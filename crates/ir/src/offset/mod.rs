mod ops;

use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tap::prelude::*;
use vmm_span::Walk;
use vmm_utils::GetOrZero;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Offset(pub isize);

impl Offset {
	#[must_use]
	pub const fn abs(self) -> Self {
		Self(self.0.abs())
	}

	#[must_use]
	pub const fn value(self) -> isize {
		self.0
	}

	#[must_use]
	pub const fn new(value: isize) -> Self {
		Self(value)
	}
}

impl<'de> Deserialize<'de> for Offset {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		isize::deserialize(deserializer).map(Self::new)
	}
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let alt = f.alternate();

		if alt {
			f.write_char('[')?;
		}

		Display::fmt(&self.value(), f)?;

		if alt {
			f.write_char(']')?;
		}

		Ok(())
	}
}

impl From<&Self> for Offset {
	fn from(value: &Self) -> Self {
		*value
	}
}

impl From<isize> for Offset {
	fn from(value: isize) -> Self {
		Self::new(value)
	}
}

impl From<&isize> for Offset {
	fn from(value: &isize) -> Self {
		(*value).convert::<Self>()
	}
}

impl GetOrZero<Self> for Offset {
	fn get_or_zero(self) -> Self {
		self
	}
}

impl GetOrZero<Offset> for Option<Offset> {
	fn get_or_zero(self) -> Offset {
		match self {
			Some(offset) => offset,
			None => Offset::new(0),
		}
	}
}

impl PartialEq<isize> for Offset {
	fn eq(&self, other: &isize) -> bool {
		PartialEq::eq(&self.0, other)
	}
}

impl PartialEq<Offset> for isize {
	fn eq(&self, other: &Offset) -> bool {
		PartialEq::eq(self, &other.0)
	}
}

impl PartialOrd<isize> for Offset {
	fn partial_cmp(&self, other: &isize) -> Option<Ordering> {
		Some(self.0.cmp(other))
	}
}

impl PartialOrd<Offset> for isize {
	fn partial_cmp(&self, other: &Offset) -> Option<Ordering> {
		Some(self.cmp(&other.0))
	}
}

impl Serialize for Offset {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Serialize::serialize(&self.0, serializer)
	}
}

impl Walk for Offset {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		Walk::steps_between(&start.0, &end.0)
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		Some(Self(Walk::forward_checked(start.0, count)?))
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		Some(Self(Walk::backward_checked(start.0, count)?))
	}
}
