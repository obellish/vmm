use vmm_goap_core::{
	prelude::*,
	simple::{
		simple_action, simple_decrement_action, simple_increment_action, simple_multi_mutate_action,
	},
};

#[test]
fn basic_bool_setting() {
	let start = LocalState::new().add_datum("is_hungry", Datum::Bool(true));

	let goal = Goal::new().add_requirement("is_hungry", Compare::Equals(Datum::Bool(false)));

	let eat_mutator = Mutator::Set("is_hungry".to_owned(), Datum::Bool(false));

	let eat_consequence = Effect::new("eat").add_mutator(eat_mutator.clone());

	let eat_action = Action::new("eat").add_effect(eat_consequence);

	let actions = [eat_action];

	let plan = Node::to_effects(start.make_plan(&actions, &goal).unwrap().0).collect::<Vec<_>>();
	assert_eq!(plan.len(), 1);

	let cons = plan.first().unwrap();
	assert_eq!(cons.action, "eat");
	assert_eq!(cons.mutators.len(), 1);
	assert_eq!(cons.mutators.first().unwrap().clone(), eat_mutator);

	let expected_state = LocalState::new().add_datum("is_hungry", Datum::Bool(false));

	assert_eq!(cons.state, expected_state);
}

#[test]
fn no_actions_needed() {
	let start = LocalState::new().add_datum("is_hungry", false);

	let goal = Goal::new().add_requirement("is_hungry", Compare::Equals(false.into()));

	let eat_mutator = Mutator::Set("is_hungry".to_owned(), false.into());

	let eat_consequence = Effect::new("eat").add_mutator(eat_mutator);

	let eat_action = Action::new("eat").add_effect(eat_consequence);

	let actions = [eat_action];

	let (plan, plan_cost) = start.make_plan(&actions, &goal).unwrap();

	assert_eq!(plan.len(), 1);
	assert_eq!(plan_cost, 0);

	let expected_state = LocalState::new().add_datum("is_hungry", false);
	assert_eq!(plan.first().unwrap().state(), &expected_state);
}

#[test]
fn simple_action_test() {
	let start = LocalState::new().add_datum("is_hungry", true);
	let expected_state = LocalState::new().add_datum("is_hungry", false);

	let goal = Goal::new().add_requirement("is_hungry", Compare::Equals(false.into()));

	let eat_action = simple_action("eat", "is_hungry", false);
	let eat_mutator = Mutator::Set("is_hungry".to_owned(), false.into());

	let actions = [eat_action];

	let plan = Node::to_effects(start.make_plan(&actions, &goal).unwrap().0).collect::<Vec<_>>();

	assert_eq!(plan.len(), 1);

	let cons = plan.first().unwrap();
	assert_eq!(cons.action, "eat");
	assert_eq!(cons.mutators.len(), 1);
	assert_eq!(cons.mutators.first().unwrap(), &eat_mutator);
	assert_eq!(cons.state, expected_state);
}

#[test]
fn two_bools() {
	let start = LocalState::new()
		.add_datum("is_hungry", true)
		.add_datum("is_tired", true);

	let expected_state = LocalState::new()
		.add_datum("is_hungry", false)
		.add_datum("is_tired", false);

	let goal = Goal::new()
		.add_requirement("is_hungry", Compare::Equals(false.into()))
		.add_requirement("is_tired", Compare::Equals(false.into()));

	let eat_action = simple_action("eat", "is_hungry", false);
	let sleep_action = simple_action("sleep", "is_tired", false);

	let actions = [eat_action, sleep_action];
	let plan = start.make_plan(&actions, &goal).unwrap();

	let cons = Node::to_effects(plan.0).collect::<Vec<_>>();
	assert_eq!(cons.len(), 2);

	let first_cons = cons.first().unwrap();
	assert_eq!(first_cons.action, "eat");
	assert_eq!(first_cons.mutators.len(), 1);

	let last_cons = cons.last().unwrap();
	assert_eq!(last_cons.action, "sleep");
	assert_eq!(last_cons.mutators.len(), 1);

	assert_eq!(last_cons.state, expected_state);
}
