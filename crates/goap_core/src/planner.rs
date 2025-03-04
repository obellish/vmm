use std::fmt::{Debug, Formatter, Result as FmtResult};

use bevy_reflect::Reflect;

use super::{Action, Effect, Goal, LocalState};

#[derive(Clone, PartialEq, Eq, Hash, Reflect)]
pub enum Node {
	Effect(Effect),
	State(LocalState),
}

impl Node {
	#[must_use]
	pub const fn state(&self) -> &LocalState {
		match self {
			Self::State(state) | Self::Effect(Effect { state, .. }) => state,
		}
	}

	fn heuristic(&self, goal: &Goal) -> usize {
		self.state().distance_to_goal(goal) as usize
	}

	fn successors<'a>(&'a self, actions: &'a [Action]) -> impl Iterator<Item = (Self, usize)> + 'a {
		let state = self.state();
		actions.iter().filter_map(move |action| {
			if state.check_preconditions(action) && !action.effects.is_empty() {
				let new_state = state.clone();
				let first_effect = &action.effects[0];

				let mut new_data = new_state.data;
				for mutator in &first_effect.mutators {
					mutator.apply(&mut new_data);
				}

				let new_effect = Effect {
					action: first_effect.action.clone(),
					mutators: first_effect.mutators.clone(),
					cost: first_effect.cost,
					state: LocalState { data: new_data },
				};

				Some((Self::Effect(new_effect), first_effect.cost))
			} else {
				None
			}
		})
	}

	fn is_goal(&self, goal: &Goal) -> bool {
		goal.requirements.iter().all(|(key, value)| {
			self.state().data.get(key).map_or_else(
				|| panic!("couldn't find key {key} in LocalState"),
				|state_value| value.compare_to(state_value),
			)
		})
	}

	pub fn to_effects(plan: impl IntoIterator<Item = Self>) -> impl Iterator<Item = Effect> {
		plan.into_iter().filter_map(|node| match node {
			Self::Effect(e) => Some(e),
			Self::State(_) => None,
		})
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Effect(e) => Debug::fmt(&e, f),
			Self::State(s) => Debug::fmt(&s, f),
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlanningStrategy {
	#[default]
	ToGoal,
}

impl PlanningStrategy {
	#[must_use]
	pub fn make_plan(
		self,
		start: &LocalState,
		actions: &[Action],
		goal: &Goal,
	) -> Option<(Vec<Node>, usize)> {
		match self {
			Self::ToGoal => {
				let start_node = Node::State(start.clone());
				pathfinding::directed::astar::astar(
					&start_node,
					|node| node.successors(actions).collect::<Vec<_>>().into_iter(),
					|node| node.heuristic(goal),
					|node| node.is_goal(goal),
				)
			}
		}
	}
}
