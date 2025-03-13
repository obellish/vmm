use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::{LocalState, Mutator};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Effect<K: Ord = String> {
	pub action: String,
	pub mutators: Vec<Mutator<K>>,
	pub state: LocalState<K>,
	pub cost: usize,
}

impl<K: Ord> Effect<K> {
	pub fn new(action: impl Into<String>) -> Self {
		Self {
			action: action.into(),
			mutators: Vec::new(),
			state: LocalState::new(),
			cost: 1,
		}
	}

	pub fn push_mutator(&mut self, mutator: Mutator<K>) {
		self.mutators.push(mutator);
	}

	#[must_use]
	pub fn with_mutator(mut self, mutator: Mutator<K>) -> Self {
		self.push_mutator(mutator);
		self
	}
}

impl<K> Hash for Effect<K>
where
	K: Hash + Ord,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.action.hash(state);
		self.mutators.hash(state);
		self.state.hash(state);
	}
}
