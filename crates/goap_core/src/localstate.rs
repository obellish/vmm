use std::{
	collections::BTreeMap,
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
};

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::{Action, Datum, Goal};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
pub struct LocalState<K: Ord = String> {
	pub data: InternalData<K>,
}

impl<K: Ord> LocalState<K> {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			data: InternalData::new(),
		}
	}

	#[must_use]
	pub fn with_datum(mut self, key: K, datum: impl Into<Datum>) -> Self {
		self.insert(key.into(), datum.into());
		self
	}

	#[must_use]
	pub fn distance_to_goal(&self, goal: &Goal<K>) -> u64 {
		goal.requirements
			.iter()
			.map(|(key, goal_value)| match self.get(key) {
				Some(state_value) => state_value.distance(&goal_value.value()),
				None => 1,
			})
			.sum()
	}

	#[must_use]
	pub fn check_preconditions(&self, action: &Action<K>) -> bool {
		action.preconditions.iter().all(|(key, value)| {
			let Some(state_value) = self.get(key) else {
				return false;
			};

			value.compare_to(state_value)
		})
	}
}

impl<K: Ord> Deref for LocalState<K> {
	type Target = InternalData<K>;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<K: Ord> DerefMut for LocalState<K> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

impl<K> Hash for LocalState<K>
where
	K: Hash + Ord,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.len().hash(state);
		for (k, v) in &self.data {
			k.hash(state);
			v.hash(state);
		}
	}
}

pub type InternalData<K> = BTreeMap<K, Datum>;

#[cfg(test)]
mod tests {
	use crate::{Action, Compare, Goal, LocalState};

	#[test]
	fn distance_to_goal_no_action() {
		let state = LocalState::new().with_datum("energy", 50i64);
		let goal_state = Goal::new().with_requirement("energy", Compare::equals(50i64));
		let distance = state.distance_to_goal(&goal_state);
		assert_eq!(distance, 0);
	}

	#[test]
	fn distance_to_goal() {
		let state = LocalState::new().with_datum("energy", 25i64);
		let goal_state = Goal::new().with_requirement("energy", Compare::equals(50i64));
		let distance = state.distance_to_goal(&goal_state);
		assert_eq!(distance, 25);
	}

	#[test]
	fn distance_to_multi_goal() {
		let state = LocalState::new()
			.with_datum("energy", 25i64)
			.with_datum("hunger", 25.0);

		let goal = Goal::new()
			.with_requirement("energy", Compare::equals(50i64))
			.with_requirement("hunger", Compare::equals(50.0));

		let distance = state.distance_to_goal(&goal);

		assert_eq!(distance, 50);
	}

	#[test]
	fn check_preconditions_empty() {
		let state = LocalState::new().with_datum("is_hungry", false);
		let action = Action::default();

		let result = state.check_preconditions(&action);
		assert!(result);
	}

	#[test]
	fn check_preconditions_true() {
		let state = LocalState::new().with_datum("is_hungry", true);
		let action = Action::new("eat").with_precondition("is_hungry", Compare::equals(false));

		let result = state.check_preconditions(&action);
		assert!(!result);
	}

	#[test]
	fn check_preconditions_conflicting() {
		let state = LocalState::new().with_datum("is_hungry", true);
		let action = Action::new("eat")
			.with_precondition("is_hungry", Compare::equals(false))
			.with_precondition("is_hungry", Compare::equals(true));

		assert!(!state.check_preconditions(&action));

		let action = Action::new("eat")
			.with_precondition("is_hungry", Compare::equals(true))
			.with_precondition("is_hungry", Compare::equals(false));

		assert!(!state.check_preconditions(&action));
	}
}
