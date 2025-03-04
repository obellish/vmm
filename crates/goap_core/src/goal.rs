use std::{
	collections::BTreeMap,
	hash::{Hash, Hasher},
};

use bevy_reflect::prelude::*;

use super::Compare;

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
#[repr(transparent)]
pub struct Goal {
	pub requirements: BTreeMap<String, Compare>,
}

impl Goal {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			requirements: BTreeMap::new(),
		}
	}

	#[must_use]
	pub fn add_requirement(mut self, key: impl Into<String>, value: Compare) -> Self {
		self.insert_requirement(key, value);
		self
	}

	pub fn insert_requirement(
		&mut self,
		key: impl Into<String>,
		value: Compare,
	) -> Option<Compare> {
		self.requirements.insert(key.into(), value)
	}
}

impl Default for Goal {
	fn default() -> Self {
		Self::new()
	}
}

impl FromIterator<(String, Compare)> for Goal {
	fn from_iter<T: IntoIterator<Item = (String, Compare)>>(iter: T) -> Self {
		let mut goal = Self::new();
		for (key, value) in iter {
			goal.insert_requirement(key, value);
		}

		goal
	}
}

impl Hash for Goal {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.requirements.len().hash(state);
		for (key, value) in &self.requirements {
			key.hash(state);
			value.hash(state);
		}
	}
}
