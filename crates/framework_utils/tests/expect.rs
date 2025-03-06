use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_framework_utils::expect::Expect;

#[derive(Component)]
struct A;

#[derive(Component)]
struct B;

#[test]
#[expect(clippy::should_panic_without_expect)]
#[should_panic]
fn expected_component() {
	let mut w = World::new();
	w.spawn(A);
	w.run_system_once(|q: Query<'_, '_, (&A, Expect<&B>)>| for _ in q.iter() {})
		.unwrap();
}
