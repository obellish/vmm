use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::Hash,
};

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use super::{Action, Effect, Goal, LocalState};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum Node<K: Ord = String> {
	Effect(Effect<K>),
	State(LocalState<K>),
}

impl<K: Ord> Node<K> {
	#[must_use]
	pub const fn state(&self) -> &LocalState<K> {
		match self {
			Self::State(state) | Self::Effect(Effect { state, .. }) => state,
		}
	}

    pub fn effects_from_plan(plan: impl IntoIterator<Item = Self>) -> impl Iterator<Item = Effect<K>> {
        plan.into_iter().filter_map(|n| {
            if let Self::Effect(e) = n {
                Some(e)
            } else {
                None
            }
        })
    }
}

impl<K> Debug for Node<K>
where
	K: Debug + Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Effect(e) => Debug::fmt(&e, f),
			Self::State(s) => Debug::fmt(&s, f),
		}
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub enum PlanningStrategy {
	#[default]
	StartToGoal,
}

impl PlanningStrategy {
	#[must_use]
	pub fn make_plan<K>(
		self,
		start: &LocalState<K>,
		actions: &[Action<K>],
		goal: &Goal<K>,
	) -> Option<(Vec<Node<K>>, usize)>
	where
		K: Clone + Hash + Ord,
	{
		match self {
			Self::StartToGoal => {
				let start_node = Node::State(start.clone());
				goap_pathfinding::directed::astar::astar(
					&start_node,
					|node| successors(node, actions).collect::<Vec<_>>(),
					|node| node.state().distance_to_goal(goal) as usize,
					|node| is_goal(node, goal),
				)
			}
		}
	}
}

fn successors<'a, K>(
	node: &'a Node<K>,
	actions: &'a [Action<K>],
) -> impl Iterator<Item = (Node<K>, usize)> + 'a
where
	K: Clone + Ord,
{
	let state = node.state();
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

			Some((Node::Effect(new_effect), first_effect.cost))
		} else {
			None
		}
	})
}

fn is_goal<K: Ord>(node: &Node<K>, goal: &Goal<K>) -> bool {
	goal.requirements.iter().all(|(key, value)| {
		let Some(state_value) = node.state().get(key) else {
			panic!("couldn't find key in state");
		};

		value.compare_to(state_value)
	})
}
