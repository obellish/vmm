use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;

use super::{Compare, Effect, Mutator};

#[derive(Debug, Default, Clone, PartialEq, Eq, Reflect)]
pub struct Action {
	pub key: String,
	pub preconditions: Vec<(String, Compare)>,
	pub effects: Vec<Effect>,
}

impl Action {
	#[must_use]
	pub fn new(key: impl Into<String>) -> Self {
		Self {
			key: key.into(),
			preconditions: Vec::new(),
			effects: Vec::new(),
		}
	}

	#[must_use]
	pub fn add_precondition(mut self, key: impl Into<String>, value: Compare) -> Self {
		self.push_precondition(key, value);
		self
	}

	pub fn push_precondition(&mut self, key: impl Into<String>, value: Compare) {
		self.preconditions.push((key.into(), value));
	}

	#[must_use]
	pub fn add_effect(mut self, effect: Effect) -> Self {
		self.push_effect(effect);
		self
	}

	pub fn push_effect(&mut self, effect: Effect) {
		self.effects.push(effect);
	}

	#[must_use]
	pub fn add_mutator(mut self, mutator: Mutator) -> Self {
		self.push_mutator(mutator);
		self
	}

	pub fn push_mutator(&mut self, mutator: Mutator) {
		if matches!(self.effects.len(), 0) {
			self.effects = vec![Effect::new(self.key.clone()).add_mutator(mutator)];
		} else {
			let effect = &mut self.effects[0];
			effect.mutators.push(mutator);
		}
	}

	pub fn set_cost(&mut self, new_cost: usize) {
		let effect = &mut self.effects[0];
		effect.cost = new_cost;
	}
}

impl Hash for Action {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.key.hash(state);
		self.preconditions.hash(state);
		self.effects.hash(state);
	}
}
