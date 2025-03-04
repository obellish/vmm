use super::{Action, Datum, Effect, LocalState};
use crate::Mutator;

pub fn simple_action(
	name: impl Into<String>,
	key_to_mutate: impl Into<String>,
	from_value: impl Into<Datum>,
) -> Action {
	simple_multi_mutate_action(name, [(key_to_mutate.into(), from_value)])
}

pub fn simple_multi_mutate_action<T>(
	name: impl Into<String>,
	muts: impl IntoIterator<Item = (String, T)>,
) -> Action
where
	T: Into<Datum>,
{
	let name = name.into();
	let mut mutators = Vec::new();

	for m in muts {
		mutators.push(Mutator::Set(m.0, m.1.into()));
	}

	Action {
		key: String::clone(&name),
		preconditions: Vec::new(),
		effects: vec![Effect {
			action: name,
			mutators,
			state: LocalState::new(),
			cost: 1,
		}],
	}
}

pub fn simple_increment_action<T>(name: impl Into<String>, key_to_mutate: impl Into<String>, from_value: T) -> Action
where
	T: Into<Datum>,
{
	let name: String = name.into();
	let mut action = simple_multi_mutate_action::<T>(name.clone(), []);
	action.effects = vec![Effect {
		action: name,
		mutators: vec![Mutator::Increment(key_to_mutate.into(), from_value.into())],
		state: LocalState::new(),
		cost: 1,
	}];

	action
}

pub fn simple_decrement_action<T>(name: impl Into<String>, key_to_mutate: impl Into<String>, from_value: T) -> Action
where
	T: Into<Datum>,
{
	let name: String = name.into();
	let mut action = simple_multi_mutate_action::<T>(name.clone(), []);
	action.effects = vec![Effect {
		action: name,
		mutators: vec![Mutator::Decrement(key_to_mutate.into(), from_value.into())],
		state: LocalState::new(),
		cost: 1,
	}];

	action
}
