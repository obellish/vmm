use criterion::{Criterion, criterion_group, criterion_main};
use vmm_goap_core::{
	prelude::*,
	simple::{simple_decrement_action, simple_increment_action},
};

fn long_plan(strategy: PlanningStrategy) {
	let start = LocalState::new()
		.add_datum("energy", 30i64)
		.add_datum("hunger", 70i64)
		.add_datum("gold", 0i64);

	let expected_state = LocalState::new()
		.add_datum("energy", 50i64)
		.add_datum("hunger", 50i64)
		.add_datum("gold", 7i64);

	let goal = Goal::new().add_requirement("gold", Compare::Equals(7i64.into()));

	let sleep_action = simple_increment_action("sleep", "energy", 10i64);
	let eat_action = simple_decrement_action("eat", "hunger", 10i64)
		.add_precondition("energy", Compare::GreaterThanEquals(25i64.into()));

	let rob_people = simple_increment_action("rob", "gold", 1i64)
		.add_effect(Effect {
			action: "rob".to_owned(),
			mutators: vec![
				Mutator::Decrement("energy".to_owned(), 5i64.into()),
				Mutator::Increment("hunger".to_owned(), 5i64.into()),
			],
			state: LocalState::new(),
			cost: 1,
		})
		.add_precondition("hunger", Compare::LessThanEquals(50i64.into()))
		.add_precondition("energy", Compare::GreaterThanEquals(50i64.into()));

	let actions = [sleep_action, eat_action, rob_people];

	let plan = strategy.make_plan(&start, &actions, &goal);
	let effects = Node::to_effects(plan.unwrap().0).collect::<Vec<_>>();

	assert_eq!(effects.len(), 11);
	assert_eq!(effects.last().unwrap().state, expected_state);
}

fn bench_long_plan(c: &mut Criterion) {
	c.bench_function("long_plan", |b| {
		b.iter(|| long_plan(PlanningStrategy::ToGoal));
	});
}

criterion_group!(benches, bench_long_plan);
criterion_main!(benches);
