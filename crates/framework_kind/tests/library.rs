use bevy_ecs::{
	prelude::*,
	system::{RunSystemError, RunSystemOnce},
};
use bevy_framework_kind::{Any, prelude::*};

#[derive(Component)]
struct Foo;

struct NotFoo;

impl Kind for NotFoo {
	type Filter = Without<Foo>;
}

#[derive(Component)]
struct Bar;

kind!(Foo is Bar);

fn count<T: Kind>(query: Query<'_, '_, Instance<T>>) -> usize {
	query.iter().count()
}

#[test]
fn kind_with() -> Result<(), RunSystemError> {
	let mut world = World::new();
	world.spawn(Foo);

	assert_eq!(world.run_system_once(count::<Foo>)?, 1);

	Ok(())
}

#[test]
fn kind_without() -> Result<(), RunSystemError> {
	let mut world = World::new();
	world.spawn(Foo);

	assert_eq!(world.run_system_once(count::<NotFoo>)?, 0);

	Ok(())
}

#[test]
fn kind_multi() -> Result<(), RunSystemError> {
	let mut world = World::new();
	world.spawn((Foo, Bar));

	assert_eq!(world.run_system_once(count::<Foo>)?, 1);
	assert_eq!(world.run_system_once(count::<Bar>)?, 1);

	Ok(())
}

#[test]
fn kind_cast() {
	let any = Instance::<Any>::PLACEHOLDER;
	let foo = Instance::<Foo>::PLACEHOLDER;
	let bar = foo.cast_into::<Bar>();

	assert_eq!(foo.cast_into_any(), any);
	assert_eq!(bar.cast_into_any(), any);

	assert_eq!(foo.entity(), bar.entity());
}
