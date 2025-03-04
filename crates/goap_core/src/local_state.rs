use std::{
	collections::BTreeMap,
	hash::{Hash, Hasher},
};

use bevy_reflect::Reflect;

use super::{Action, Datum, Goal};
use crate::planner::{Node, PlanningStrategy};

pub type InternalData = BTreeMap<String, Datum>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Reflect)]
#[repr(transparent)]
pub struct LocalState {
	pub data: InternalData,
}

impl LocalState {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			data: InternalData::new(),
		}
	}

	#[must_use]
	pub fn add_datum(mut self, key: impl Into<String>, value: impl Into<Datum>) -> Self {
		self.insert_datum(key, value);
		self
	}

	pub fn insert_datum(
		&mut self,
		key: impl Into<String>,
		value: impl Into<Datum>,
	) -> Option<Datum> {
		self.data.insert(key.into(), value.into())
	}

	#[must_use]
	pub fn check_preconditions(&self, action: &Action) -> bool {
		action.preconditions.iter().all(|(key, value)| {
			let state_value = self
				.data
				.get(key)
				.unwrap_or_else(|| panic!("couldn't find key {key} is LocalState"));

			value.compare_to(state_value)
		})
	}

	#[must_use]
	pub fn distance_to_goal(&self, goal: &Goal) -> u64 {
		goal.requirements
			.iter()
			.map(|(key, goal_value)| {
				self.data
					.get(key)
					.map_or(1, |state_value| state_value.distance(&goal_value.value()))
			})
			.sum()
	}

	#[must_use]
	pub fn make_plan(&self, actions: &[Action], goal: &Goal) -> Option<(Vec<Node>, usize)> {
		PlanningStrategy::ToGoal.make_plan(self, actions, goal)
	}
}

impl Hash for LocalState {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.data.len().hash(state);
		for (key, value) in &self.data {
			key.hash(state);
			value.hash(state);
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{Action, Compare, Datum, Goal, LocalState};

	#[test]
	fn check_preconditions_empty() {
		let state = LocalState::new().add_datum("is_hungry".to_owned(), Datum::Bool(true));
		let action = Action::default();

		let result = state.check_preconditions(&action);
		assert!(result);
	}

	#[test]
	fn check_preconditions_false() {
		let state = LocalState::new().add_datum("is_hungry".to_owned(), Datum::Bool(true));
		let action = Action::default()
			.add_precondition("is_hungry".to_owned(), Compare::Equals(Datum::Bool(false)));

		let result = state.check_preconditions(&action);
		assert!(!result);
	}

	#[test]
	fn check_preconditions_conflicting() {
		let state = LocalState::new().add_datum("is_hungry".to_owned(), Datum::Bool(true));

		let action = Action::default()
			.add_precondition("is_hungry".to_owned(), Compare::Equals(Datum::Bool(false)))
			.add_precondition("is_hungry".to_owned(), Compare::Equals(Datum::Bool(true)));

		let result = state.check_preconditions(&action);
		assert!(!result);

		let action = Action::default()
			.add_precondition("is_hungry".to_owned(), Compare::Equals(Datum::Bool(true)))
			.add_precondition("is_hungry".to_owned(), Compare::Equals(Datum::Bool(false)));

		let result = state.check_preconditions(&action);
		assert!(!result);
	}

	#[test]
	fn distance_to_goal() {
		let state = LocalState::new().add_datum("energy".to_owned(), Datum::I64(50));
		let goal_state =
			Goal::new().add_requirement("energy".to_owned(), Compare::Equals(Datum::I64(50)));
		let distance = state.distance_to_goal(&goal_state);

		assert_eq!(distance, 0);

		let state = LocalState::new().add_datum("energy".to_owned(), Datum::I64(25));
		let goal_state =
			Goal::new().add_requirement("energy".to_owned(), Compare::Equals(Datum::I64(50)));
		let distance = state.distance_to_goal(&goal_state);
		assert_eq!(distance, 25);

		let state = LocalState::new()
			.add_datum("energy".to_owned(), Datum::I64(25))
			.add_datum("hunger".to_owned(), Datum::F64(25.0));

		let goal_state = Goal::new()
			.add_requirement("energy".to_owned(), Compare::Equals(Datum::I64(50)))
			.add_requirement("hunger".to_owned(), Compare::Equals(Datum::F64(50.0)));

		let distance = state.distance_to_goal(&goal_state);

		assert_eq!(distance, 50);
	}
}
