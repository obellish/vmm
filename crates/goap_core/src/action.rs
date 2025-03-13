use std::hash::{Hash, Hasher};

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::{Compare, Effect, Mutator};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Action<K: Ord = String> {
	pub key: String,
	pub preconditions: Vec<(K, Compare)>,
	pub effects: Vec<Effect<K>>,
}

impl<K> Action<K>
where
	K: Clone + Ord,
{
	pub fn new(key: impl Into<String>) -> Self {
		Self {
			key: key.into(),
			preconditions: Vec::new(),
			effects: Vec::new(),
		}
	}

	pub fn push_precondition(&mut self, key: K, compare: Compare) {
		self.preconditions.push((key, compare));
	}

	#[must_use]
	pub fn with_precondition(mut self, key: K, compare: Compare) -> Self {
		self.push_precondition(key, compare);
		self
	}

	pub fn push_effect(&mut self, effect: Effect<K>) {
		self.effects.push(effect);
	}

	#[must_use]
	pub fn with_effect(mut self, effect: Effect<K>) -> Self {
		self.push_effect(effect);
		self
	}

	pub fn push_mutator(&mut self, mutator: Mutator<K>) {
		if self.effects.is_empty() {
			self.effects = vec![Effect::new(self.key.clone()).with_mutator(mutator)];
		} else {
			let effect = &mut self.effects[0];
			effect.push_mutator(mutator);
		}
	}

	#[must_use]
	pub fn with_mutator(mut self, mutator: Mutator<K>) -> Self {
		self.push_mutator(mutator);
		self
	}

	#[must_use]
	pub fn with_cost(mut self, cost: usize) -> Self {
		let effect = &mut self.effects[0];
		effect.cost = cost;

		self
	}
}

impl<K> Hash for Action<K>
where
	K: Hash + Ord,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.key.hash(state);
		self.preconditions.hash(state);
		self.effects.hash(state);
	}
}
