use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;

use super::Datum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Compare {
	Equals(Datum),
	NotEquals(Datum),
	GreaterThanEquals(Datum),
	LessThanEquals(Datum),
}

impl Compare {
	#[must_use]
	pub const fn value(self) -> Datum {
		match self {
			Self::Equals(v)
			| Self::NotEquals(v)
			| Self::GreaterThanEquals(v)
			| Self::LessThanEquals(v) => v,
		}
	}

	#[must_use]
	pub fn compare_to(&self, value: &Datum) -> bool {
		match self {
			Self::Equals(v) => value == v,
			Self::NotEquals(v) => value != v,
			Self::GreaterThanEquals(v) => value >= v,
			Self::LessThanEquals(v) => value <= v,
		}
	}
}

impl Hash for Compare {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		self.value().hash(state);
	}
}

#[cfg(test)]
mod tests {
	use crate::{Compare, Datum};

	#[test]
	fn greater_than_equals() {
		let cases = [(10, 10, true), (10, 9, false), (11, 10, false)];

		for (left, right, expected) in cases {
			let ret = Compare::GreaterThanEquals(Datum::I64(left)).compare_to(&Datum::I64(right));

			assert_eq!(
				ret, expected,
				"expected {left} to be greater than or equal to {right}, but compare_to returned {ret}"
			);
		}
	}

	#[test]
	fn less_than_equals() {
		let cases = [(10, 10, true), (10, 9, true), (11, 10, true)];

		for (left, right, expected) in cases {
			let ret = Compare::LessThanEquals(Datum::I64(left)).compare_to(&Datum::I64(right));

			assert_eq!(
				ret, expected,
				"expected {left} to be less than or equal to {right}, but compare_to returned {ret}"
			);
		}
	}

	#[test]
	fn not_equals() {
		let cases = [(10, 10, false), (10, 9, true), (11, 10, true)];

		for (left, right, expected) in cases {
			let ret = Compare::NotEquals(Datum::I64(left)).compare_to(&Datum::I64(right));

			assert_eq!(
				ret, expected,
				"expected {left} to not be equal to {right}, but compare_to returned {ret}"
			);
		}
	}
}
