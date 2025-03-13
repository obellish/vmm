use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::Datum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum Compare {
	Equals(Datum),
	NotEquals(Datum),
	GreaterThanEquals(Datum),
	LessThanEquals(Datum),
}

impl Compare {
	#[must_use]
	#[expect(clippy::trivially_copy_pass_by_ref)]
	pub fn compare_to(&self, value: &Datum) -> bool {
		match self {
			Self::Equals(v) => v == value,
			Self::NotEquals(v) => v != value,
			Self::GreaterThanEquals(v) => value >= v,
			Self::LessThanEquals(v) => value <= v,
		}
	}

	#[must_use]
	pub const fn value(self) -> Datum {
		match self {
			Self::Equals(d)
			| Self::NotEquals(d)
			| Self::GreaterThanEquals(d)
			| Self::LessThanEquals(d) => d,
		}
	}

	pub fn equals(datum: impl Into<Datum>) -> Self {
		Self::Equals(datum.into())
	}

	pub fn not_equals(datum: impl Into<Datum>) -> Self {
		Self::NotEquals(datum.into())
	}

	pub fn greater_than_equals(datum: impl Into<Datum>) -> Self {
		Self::GreaterThanEquals(datum.into())
	}

	pub fn less_than_equals(datum: impl Into<Datum>) -> Self {
		Self::LessThanEquals(datum.into())
	}
}

impl Hash for Compare {
	fn hash<H: Hasher>(&self, state: &mut H) {
		std::mem::discriminant(self).hash(state);
		match self {
			Self::Equals(d)
			| Self::NotEquals(d)
			| Self::GreaterThanEquals(d)
			| Self::LessThanEquals(d) => d.hash(state),
		}
	}
}
