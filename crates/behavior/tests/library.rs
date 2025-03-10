use anyhow::Result;
use bevy::{ecs::system::RunSystemOnce as _, prelude::*};
use bevy_behavior::prelude::*;

use self::T::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Component, Reflect)]
enum T {
	#[default]
	A,
	B,
	C,
	D,
}

impl Behavior for T {
	fn allows_next(&self, next: &Self) -> bool {
		matches!((self, next), (A, B | D) | (B, C))
	}
}

fn app() -> App {
	let mut app = App::new();
	app.add_plugins((MinimalPlugins, BehaviorPlugin::<T>::default()))
		.add_systems(Update, transition::<T>);

	app
}

#[test]
fn initial() -> Result<()> {
	let mut app = app();
	app.world_mut().spawn(A);
	app.update();
	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| { *q.single() })?,
		A
	);

	Ok(())
}

#[test]
fn push() -> Result<()> {
	let mut app = app();
	app.world_mut().spawn((A, Transition::Next(B)));
	app.update();
	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| {
				let behavior = q.single();
				(behavior.previous().copied(), *behavior.current())
			})?,
		(Some(A), B)
	);

	Ok(())
}

#[test]
fn push_reject() -> Result<()> {
	let mut app = app();
	app.world_mut().spawn((A, Transition::Next(C)));
	app.update();
	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| {
				let behavior = q.single();
				(behavior.previous().copied(), *behavior.current())
			})?,
		(None, A)
	);

	Ok(())
}

#[test]
fn pop() -> Result<()> {
	let mut app = app();
	app.world_mut().spawn((A, Transition::Next(B)));
	app.update();

	app.world_mut()
		.run_system_once(|mut q: Query<'_, '_, BehaviorMut<T>>| {
			q.single_mut().stop();
		})?;
	app.update();

	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| {
				let behavior = q.single();
				(behavior.previous().copied(), *behavior.current())
			})?,
		(None, A)
	);

	Ok(())
}

#[test]
fn pop_initial() -> Result<()> {
	let mut app = app();
	app.world_mut().spawn((A, Transition::<T>::Previous));
	app.update();
	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| { *q.single() })?,
		A
	);

	Ok(())
}

#[test]
fn sequence() -> Result<()> {
	let mut app = app();
	app.world_mut().spawn((A, Sequence::from_iter([B, D])));
	app.update();

	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| {
				let behavior = q.single();
				(behavior.previous().copied(), *behavior.current())
			})?,
		(Some(A), B)
	);

	app.world_mut()
		.run_system_once(|mut q: Query<'_, '_, BehaviorMut<T>>| {
			q.single_mut().stop();
		})?;

	app.update();
	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| {
				let behavior = q.single();
				(behavior.previous().copied(), *behavior.current())
			})?,
		(Some(A), D)
	);

	app.update();
	assert_eq!(
		app.world_mut()
			.run_system_once(|q: Query<'_, '_, BehaviorRef<T>>| {
				let behavior = q.single();
				(behavior.previous().copied(), *behavior.current())
			})?,
		(Some(A), D)
	);

	Ok(())
}
