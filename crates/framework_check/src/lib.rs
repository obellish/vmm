#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod prelude;

use bevy_app::prelude::*;
use bevy_ecs::{
	prelude::*,
	query::{QueryEntityError, QueryFilter},
	system::EntityCommands,
};
use bevy_framework_kind::prelude::*;
use bevy_framework_save::load::LoadSet;
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_utils::tracing::{debug, error, warn};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct CheckSet;

#[repr(transparent)]
pub struct Fixer(Box<dyn Fix>);

impl Fixer {
	pub fn new(f: impl Fix) -> Self {
		Self(Box::new(f))
	}

	pub fn fix(&self, entity: EntityRef<'_>, commands: &mut Commands<'_, '_>) {
		self.0.fix(entity, commands);
	}
}

#[derive(QueryFilter)]
pub struct Valid(With<Checked>, Without<Invalid>);

#[derive(Component)]
struct Checked;

#[derive(Component)]
struct Invalid;

pub enum Policy {
	Invalid,
	Purge,
	Panic,
	Repair(Fixer),
}

pub trait Fix: Send + Sync + 'static {
	fn fix(&self, entity: EntityRef<'_>, commands: &mut Commands<'_, '_>);
}

impl<F> Fix for F
where
	F: Fn(EntityRef<'_>, &mut Commands<'_, '_>) + Send + Sync + 'static,
{
	fn fix(&self, entity: EntityRef<'_>, commands: &mut Commands<'_, '_>) {
		self(entity, commands);
	}
}

pub trait CheckFilter: QueryFilter + Send + Sync + 'static {}

impl<F> CheckFilter for F where F: QueryFilter + Send + Sync + 'static {}

pub trait AppExt {
	fn check<T: Kind, F: CheckFilter>(&mut self, policy: Policy) -> &mut Self;
}

impl AppExt for App {
	fn check<T: Kind, F: CheckFilter>(&mut self, policy: Policy) -> &mut Self {
		let filter_name = || bevy_framework_utils::get_short_name(std::any::type_name::<F>());

		self.add_systems(
			PreUpdate,
			(move |query: Query<'_, '_, Instance<T>, Unchecked>,
			       check: Query<'_, '_, (), F>,
			       world: &World,
			       mut commands: Commands<'_, '_>| {
				for instance in query.iter() {
					match check.get(instance.entity()) {
						Err(QueryEntityError::QueryDoesNotMatch(..)) => {
							if let Some(mut entity) = commands.get_entity(instance.entity()) {
								entity.try_insert(Checked);
								debug!("{instance:?} is valid.");
							}
							continue;
						}
						Err(QueryEntityError::NoSuchEntity(_)) => continue,
						_ => {}
					}

					match &policy {
						Policy::Invalid => {
							if let Some(mut entity) = commands.get_entity(instance.entity()) {
								entity.try_insert((Checked, Invalid));
								error!("{instance:?} is invalid: {}", filter_name());
							}
						}
						Policy::Purge => {
							if let Some(entity) = commands.get_entity(instance.entity()) {
								entity.despawn_recursive();
								error!("{instance:?} is purged: {}", filter_name());
							}
						}
						Policy::Panic => {
							panic!("{instance:?} is strictly invalid: {}", filter_name());
						}
						Policy::Repair(fixer) => {
							if let Some(mut entity) = commands.get_entity(instance.entity()) {
								entity.try_insert(Checked);
								error!("{instance:?} is invalid: {}", filter_name());

								let entity = world.entity(instance.entity());
								fixer.fix(entity, &mut commands);
								warn!("{instance:?} was repaired.");
							}
						}
					}
				}
			})
			.after(LoadSet::Load)
			.in_set(CheckSet),
		)
	}
}

#[allow(clippy::return_self_not_must_use)]
pub trait CheckAgain {
	fn check_again(self) -> Self;
}

impl CheckAgain for &mut EntityCommands<'_> {
	fn check_again(self) -> Self {
		self.remove::<Checked>().remove::<Invalid>()
	}
}

impl CheckAgain for &mut EntityWorldMut<'_> {
	fn check_again(self) -> Self {
		self.remove::<Checked>().remove::<Invalid>()
	}
}

type Unchecked = Without<Checked>;

#[must_use]
pub const fn invalid() -> Policy {
	Policy::Invalid
}

#[must_use]
pub const fn purge() -> Policy {
	Policy::Purge
}

#[must_use]
pub const fn panic() -> Policy {
	Policy::Panic
}

pub fn repair(f: impl Fix) -> Policy {
	Policy::Repair(Fixer::new(f))
}

pub fn repair_insert<T>(component: T) -> Policy
where
	T: Clone + Component,
{
	repair(
		move |entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
			commands.entity(entity.id()).insert(component.clone());
		},
	)
}

#[must_use]
pub fn repair_insert_default<T>() -> Policy
where
	T: Component + Default,
{
	repair(
		move |entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
			commands.entity(entity.id()).insert(T::default());
		},
	)
}

pub fn repair_replace<T: Component, U>(component: U) -> Policy
where
	U: Clone + Component,
{
	repair(
		move |entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
			commands
				.entity(entity.id())
				.remove::<T>()
				.insert(component.clone());
		},
	)
}

#[must_use]
pub fn repair_replace_default<T: Component, U>() -> Policy
where
	U: Component + Default,
{
	repair(
		move |entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
			commands
				.entity(entity.id())
				.remove::<T>()
				.insert(U::default());
		},
	)
}

pub fn repair_replace_with<T: Component, U: Component, F>(f: F) -> Policy
where
	F: Fn(&T) -> U + Send + Sync + 'static,
{
	repair(
		move |entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
			let component = entity.get::<T>().unwrap();
			commands
				.entity(entity.id())
				.remove::<T>()
				.insert(f(component));
		},
	)
}

#[must_use]
pub fn repair_remove<T: Component>() -> Policy {
	repair(
		move |entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
			commands.entity(entity.id()).remove::<T>();
		},
	)
}

#[cfg(test)]
mod tests {
	use bevy::prelude::*;

	use super::*;

	#[derive(Component)]
	struct Foo;

	#[derive(Component)]
	struct Bar;

	fn app(policy: Policy) -> App {
		let mut app = App::new();
		app.add_plugins(MinimalPlugins)
			.check::<Foo, Without<Bar>>(policy);

		app
	}

	#[test]
	fn test_valid() {
		let mut app = app(panic());

		let entity = app.world_mut().spawn((Foo, Bar)).id();
		app.update();

		assert!(app.world().entity(entity).contains::<Checked>());
		assert!(!app.world().entity(entity).contains::<Invalid>());
	}

	#[test]
	fn test_invalid() {
		let mut app = app(invalid());

		let entity = app.world_mut().spawn(Foo).id();
		app.update();

		assert!(app.world().entity(entity).contains::<Checked>());
		assert!(app.world().entity(entity).contains::<Invalid>());
	}

	#[test]
	fn test_purge() {
		let mut app = app(purge());

		let entity = app.world_mut().spawn(Foo).id();
		app.update();

		assert!(app.world().get_entity(entity).is_err());
	}

	#[test]
	#[should_panic = "Foo(0v1) is strictly invalid: Without<Bar>"]
	fn test_panic() {
		let mut app = app(panic());

		app.world_mut().spawn(Foo);
		app.update();
	}

	#[test]
	fn test_repair() {
		let mut app = app(repair(
			|entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
				commands.entity(entity.id()).insert(Bar);
			},
		));

		let entity = app.world_mut().spawn(Foo).id();
		app.update();

		assert!(app.world().entity(entity).contains::<Bar>());
		assert!(app.world().entity(entity).contains::<Checked>());
	}

	#[test]
	#[should_panic = "Bar is still missing!"]
	fn test_check_again() {
		#[derive(Component)]
		struct Repaired;

		let mut app = app(repair(
			|entity: EntityRef<'_>, commands: &mut Commands<'_, '_>| {
				assert!(!entity.contains::<Repaired>(), "Bar is still missing!");

				_ = commands.entity(entity.id()).insert(Repaired).check_again();
			},
		));

		let entity = app.world_mut().spawn(Foo).id();
		app.update();

		assert!(!app.world().entity(entity).contains::<Bar>());
		assert!(!app.world().entity(entity).contains::<Checked>());

		app.update();
	}

	#[test]
	#[should_panic = "Foo(0v1) is strictly invalid: Without<Baz>"]
	fn test_multiple() {
		#[derive(Component)]
		struct Baz;

		let mut app = app(panic());
		app.check::<Foo, Without<Baz>>(panic());

		app.world_mut().spawn((Foo, Bar));
		app.update();
	}
}
