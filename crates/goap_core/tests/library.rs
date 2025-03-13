use goap_core::prelude::*;

#[test]
fn basic_bool_setting() {
	let start = LocalState::new().with_datum("is_hungry", true);

	let goal = Goal::new().with_requirement("is_hungry", Compare::equals(false));

	let eat_mutator = Mutator::set("is_hungry", false);

	let eat_consequence = Effect::new("eat").with_mutator(eat_mutator);

	let eat_action = Action::new("eat").with_effect(eat_consequence);

	let actions = [eat_action];

	let plan = Node::effects_from_plan(
		PlanningStrategy::StartToGoal
			.make_plan(&start, &actions, &goal)
			.unwrap()
			.0,
	)
	.collect::<Vec<_>>();

	assert_eq!(plan.len(), 1);

	let cons = plan.first().unwrap();
	assert_eq!(cons.action, "eat");
	assert_eq!(cons.mutators.len(), 1);
	assert_eq!(eat_mutator, cons.mutators.first().copied().unwrap());

	let expected_state = LocalState::new().with_datum("is_hungry", false);
	assert_eq!(cons.state, expected_state);
}
