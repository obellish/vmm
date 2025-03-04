use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;

use super::{LocalState, Mutator};

#[derive(Debug, Default, Clone, Reflect)]
pub struct Effect {
	pub action: String,
	pub mutators: Vec<Mutator>,
	pub state: LocalState,
	pub cost: usize,
}

impl Effect {
	pub fn new(action_name: impl Into<String>) -> Self {
		Self {
			action: action_name.into(),
			mutators: Vec::new(),
			state: LocalState::new(),
			cost: 1,
		}
	}

	#[must_use]
	pub fn add_mutator(mut self, mutator: Mutator) -> Self {
		self.push_mutator(mutator);
		self
	}

	pub fn push_mutator(&mut self, mutator: Mutator) {
		self.mutators.push(mutator);
	}
}

impl Eq for Effect {}

impl Hash for Effect {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.action.hash(state);
		self.mutators.hash(state);
		self.state.hash(state);
	}
}

impl PartialEq for Effect {
	fn eq(&self, other: &Self) -> bool {
		self.action == other.action && self.mutators == other.mutators && self.state == other.state
	}
}
