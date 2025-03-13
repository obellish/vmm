use std::{
	collections::BTreeMap,
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
};

use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};

use super::Compare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Goal<K: Ord = String> {
	pub requirements: BTreeMap<K, Compare>,
}

impl<K: Ord> Goal<K> {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			requirements: BTreeMap::new(),
		}
	}

	#[must_use]
	pub fn with_requirement(mut self, key: K, compare: impl Into<Compare>) -> Self {
		self.insert(key.into(), compare.into());
		self
	}
}

impl<K: Ord> Default for Goal<K> {
	fn default() -> Self {
		Self::new()
	}
}

impl<K: Ord> Deref for Goal<K> {
	type Target = BTreeMap<K, Compare>;

	fn deref(&self) -> &Self::Target {
		&self.requirements
	}
}

impl<K: Ord> DerefMut for Goal<K> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.requirements
	}
}

impl<K: Ord> FromIterator<(K, Compare)> for Goal<K> {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = (K, Compare)>,
	{
		Self {
			requirements: BTreeMap::from_iter(iter),
		}
	}
}

impl<K> Hash for Goal<K>
where
	K: Hash + Ord,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.len().hash(state);
		for (k, v) in &self.requirements {
			k.hash(state);
			v.hash(state);
		}
	}
}
