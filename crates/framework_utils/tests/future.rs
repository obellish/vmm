use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_framework_utils::future::{Future, FutureValue, Promise};

#[derive(Component)]
#[repr(transparent)]
struct Server(Promise<()>);

#[derive(Component)]
#[repr(transparent)]
struct Client(Future<()>);

#[test]
fn future_value() {
	let mut w = World::new();

	let (p, f) = Promise::start();
	w.spawn(Server(p));
	w.spawn(Client(f));

	assert_eq!(
		w.run_system_once(move |q: Single<'_, &Client>| q.0.poll())
			.unwrap(),
		FutureValue::Wait
	);

	assert_eq!(
		w.run_system_once(move |q: Single<'_, &Client>| { q.0.poll() })
			.unwrap(),
		FutureValue::Wait
	);

	w.run_system_once(
		move |q: Single<'_, (Entity, &Server)>, mut commands: Commands<'_, '_>| {
			let (entity, server) = *q;
			server.0.set(());
			commands.entity(entity).remove::<Server>();
		},
	)
	.unwrap();

	assert_eq!(
		w.run_system_once(move |q: Single<'_, &Client>| { q.0.poll() })
			.unwrap(),
		FutureValue::Ready(())
	);

	assert_eq!(
		w.run_system_once(move |q: Single<'_, &Client>| { q.0.poll() })
			.unwrap(),
		FutureValue::Expired
	);
}
