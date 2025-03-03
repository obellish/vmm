use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;

use super::{Datum, LocalState};

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
