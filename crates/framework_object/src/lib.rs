#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod hierarchy;
mod instance;
mod name;
pub mod prelude;
mod rebind;

use std::{
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	ops::Deref,
};

use bevy_core::Name;
use bevy_ecs::{
	prelude::*,
	query::{QueryEntityError, QueryFilter, QuerySingleError},
	system::SystemParam,
};
use bevy_framework_kind::prelude::*;
pub use bevy_framework_kind::{Any, CastInto, Kind};
use bevy_framework_utils::hierarchy::HierarchyQuery;
use bevy_hierarchy::Parent;

pub use self::{hierarchy::*, instance::*, name::*, rebind::*};

#[derive(SystemParam)]
pub struct Objects<'w, 's, T: Kind = Any, F = ()>
where
	F: QueryFilter + 'static,
{
	pub instance: Query<'w, 's, Instance<T>, F>,
	pub root: Query<'w, 's, Instance<T>, (F, Without<Parent>)>,
	pub hierarchy: HierarchyQuery<'w, 's>,
	pub name: Query<'w, 's, &'static Name>,
}

impl<'w, 's, T: Kind, F> Objects<'w, 's, T, F>
where
	F: QueryFilter + 'static,
{
	pub fn iter(&self) -> impl Iterator<Item = Object<'w, 's, '_, T>> {
		self.instance.iter().map(|instance| Object {
			instance,
			hierarchy: &self.hierarchy,
			name: &self.name,
		})
	}

	pub fn iter_root(&self) -> impl Iterator<Item = Object<'w, 's, '_, T>> {
		self.root.iter().map(|instance| Object {
			instance,
			hierarchy: &self.hierarchy,
			name: &self.name,
		})
	}

	#[must_use]
	pub fn contains(&self, entity: Entity) -> bool {
		self.instance.contains(entity)
	}

	#[must_use]
	pub fn contains_root(&self, entity: Entity) -> bool {
		self.root.contains(entity)
	}

	pub fn iter_refs<'a>(
		&'a self,
		world: &'a World,
	) -> impl Iterator<Item = ObjectRef<'w, 's, 'a, T>> {
		self.iter()
			.map(|object: Object<'_, '_, '_, T>| ObjectRef(world.entity(object.entity()), object))
	}

	pub fn iter_root_ref<'a>(
		&'a self,
		world: &'a World,
	) -> impl Iterator<Item = ObjectRef<'w, 's, 'a, T>> {
		self.iter()
			.map(|object: Object<'_, '_, '_, T>| ObjectRef(world.entity(object.entity()), object))
	}

	pub fn get(&self, entity: Entity) -> Result<Object<'w, 's, '_, T>, QueryEntityError<'_>> {
		self.instance.get(entity).map(|instance| Object {
			instance,
			hierarchy: &self.hierarchy,
			name: &self.name,
		})
	}

	pub fn get_root(&self, entity: Entity) -> Result<Object<'w, 's, '_, T>, QueryEntityError<'_>> {
		self.root.get(entity).map(|instance| Object {
			instance,
			hierarchy: &self.hierarchy,
			name: &self.name,
		})
	}

	#[must_use]
	pub fn get_ref<'a>(&'a self, entity: EntityRef<'a>) -> Option<ObjectRef<'w, 's, 'a, T>> {
		Some(ObjectRef(entity, self.get(entity.id()).ok()?))
	}

	pub fn get_single(&self) -> Result<Object<'w, 's, '_, T>, QuerySingleError> {
		self.instance.get_single().map(|instance| Object {
			instance,
			hierarchy: &self.hierarchy,
			name: &self.name,
		})
	}

	pub fn get_single_root(&self) -> Result<Object<'w, 's, '_, T>, QuerySingleError> {
		self.root.get_single().map(|instance| Object {
			instance,
			hierarchy: &self.hierarchy,
			name: &self.name,
		})
	}

	#[must_use]
	pub fn get_single_ref<'a>(&'a self, entity: EntityRef<'a>) -> Option<ObjectRef<'w, 's, 'a, T>> {
		Some(ObjectRef(entity, self.get_single().ok()?))
	}

	#[must_use]
	pub fn instance(&self, instance: Instance<T>) -> Object<'w, 's, '_, T> {
		self.get(instance.entity()).expect("instance must be valid")
	}
}

pub struct Object<'w, 's, 'a, T: Kind = Any> {
	instance: Instance<T>,
	hierarchy: &'a HierarchyQuery<'w, 's>,
	name: &'a Query<'w, 's, &'static Name>,
}

impl<'w, 's, 'a, T: Kind> Object<'w, 's, 'a, T> {
	#[must_use]
	pub const unsafe fn from_base_unchecked(base: Object<'w, 's, 'a>) -> Self {
		Self {
			instance: unsafe { base.instance.cast_into_unchecked() },
			hierarchy: base.hierarchy,
			name: base.name,
		}
	}
}

impl<'w, 's, 'a, T: Component> Object<'w, 's, 'a, T> {
	pub fn from_base(world: &World, object: Object<'w, 's, 'a>) -> Option<Self> {
		let entity = world.entity(object.entity());
		let instance = Instance::<T>::from_entity(entity)?;

		Some(object.rebind_as(instance))
	}
}

impl<T: Kind> Clone for Object<'_, '_, '_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Kind> Copy for Object<'_, '_, '_, T> {}

impl<T: Kind> Debug for Object<'_, '_, '_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&T::debug_name())?;
		f.write_char('(')?;
		Display::fmt(&self.entity(), f)?;
		if let Some(name) = self.name() {
			f.write_str(", ")?;
			f.write_str(name)?;
		}
		f.write_char(')')
	}
}

impl<T: Kind> Eq for Object<'_, '_, '_, T> {}

impl<'w, 's, 'a, T: Kind> From<ObjectRef<'w, 's, 'a, T>> for Object<'w, 's, 'a, T> {
	fn from(value: ObjectRef<'w, 's, 'a, T>) -> Self {
		value.1
	}
}

impl<'w, 's, 'a, T: Kind> From<&ObjectRef<'w, 's, 'a, T>> for Object<'w, 's, 'a, T> {
	fn from(value: &ObjectRef<'w, 's, 'a, T>) -> Self {
		(*value).into()
	}
}

impl<T: Kind> From<Object<'_, '_, '_, T>> for Entity {
	fn from(value: Object<'_, '_, '_, T>) -> Self {
		value.entity()
	}
}

impl<T: Kind> From<Object<'_, '_, '_, T>> for Instance<T> {
	fn from(value: Object<'_, '_, '_, T>) -> Self {
		value.instance()
	}
}

impl<T: Kind> PartialEq for Object<'_, '_, '_, T> {
	fn eq(&self, other: &Self) -> bool {
		self.instance == other.instance
	}
}

pub struct ObjectRef<'w, 's, 'a, T: Kind = Any>(EntityRef<'a>, Object<'w, 's, 'a, T>);

impl<'w, 's, 'a, T: Kind> ObjectRef<'w, 's, 'a, T> {
	#[must_use]
	pub fn get<U: Component>(&self) -> Option<&U> {
		self.0.get::<U>()
	}

	#[must_use]
	#[allow(clippy::trivially_copy_pass_by_ref)]
	pub fn contains<U: Component>(&self) -> bool {
		self.0.contains::<U>()
	}

	#[must_use]
	pub const unsafe fn from_base_unchecked(base: ObjectRef<'w, 's, 'a>) -> Self {
		Self(base.0, unsafe { Object::from_base_unchecked(base.1) })
	}
}

impl<T: Kind> Clone for ObjectRef<'_, '_, '_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Kind> Copy for ObjectRef<'_, '_, '_, T> {}

impl<T: Kind> Debug for ObjectRef<'_, '_, '_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.1, f)
	}
}

impl<T: Component> Deref for ObjectRef<'_, '_, '_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.get::<T>().unwrap()
	}
}

impl<T: Kind> Eq for ObjectRef<'_, '_, '_, T> {}

impl<T: Kind> From<ObjectRef<'_, '_, '_, T>> for Entity {
	fn from(value: ObjectRef<'_, '_, '_, T>) -> Self {
		value.entity()
	}
}

impl<T: Kind> From<ObjectRef<'_, '_, '_, T>> for Instance<T> {
	fn from(value: ObjectRef<'_, '_, '_, T>) -> Self {
		value.instance()
	}
}

impl<T: Kind> PartialEq for ObjectRef<'_, '_, '_, T> {
	fn eq(&self, other: &Self) -> bool {
		self.1 == other.1
	}
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)]
mod tests {
	use anyhow::Result;
	use bevy::{ecs::system::RunSystemOnce as _, prelude::*};

	use super::*;

	#[derive(Component)]
	struct T;

	#[test]
	fn find_by_path() -> Result<()> {
		let mut w = World::new();

		let (a, b, c, d) = w.run_system_once(|mut commands: Commands<'_, '_>| {
			let a = commands.spawn(Name::new("A")).id();
			let b = commands.spawn(Name::new("B")).id();
			let c = commands.spawn(Name::new("C")).id();
			let d = commands.spawn(Name::new("D")).id();

			commands.entity(a).add_children(&[b]);
			commands.entity(b).add_children(&[c, d]);

			(a, b, c, d)
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects.get(a).unwrap().find_by_path("").unwrap().entity();
			assert_eq!(a, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects.get(a).unwrap().find_by_path("B").unwrap().entity();
			assert_eq!(b, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(a)
				.unwrap()
				.find_by_path("B/C")
				.unwrap()
				.entity();
			assert_eq!(c, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(a)
				.unwrap()
				.find_by_path("B/D")
				.unwrap()
				.entity();
			assert_eq!(d, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(a)
				.unwrap()
				.find_by_path("B/*")
				.unwrap()
				.entity();
			assert_eq!(c, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(a)
				.unwrap()
				.find_by_path("*/D")
				.unwrap()
				.entity();
			assert_eq!(d, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(a)
				.unwrap()
				.find_by_path("*/*")
				.unwrap()
				.entity();
			assert_eq!(c, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects.get(b).unwrap().find_by_path("..").unwrap().entity();
			assert_eq!(a, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects.get(c).unwrap().find_by_path("..").unwrap().entity();
			assert_eq!(b, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(c)
				.unwrap()
				.find_by_path("../D")
				.unwrap()
				.entity();
			assert_eq!(d, x);
		})?;

		w.run_system_once(move |objects: Objects<'_, '_>| {
			let x = objects
				.get(c)
				.unwrap()
				.find_by_path("../C")
				.unwrap()
				.entity();
			assert_eq!(c, x);
		})?;

		Ok(())
	}

	#[test]
	fn object_ref() -> Result<()> {
		let mut w = World::new();
		let entity = w.spawn(T).id();

		assert!(
			w.run_system_once(move |world: &World, objects: Objects<'_, '_, T>| {
				objects
					.get_single_ref(world.entity(entity))
					.unwrap()
					.contains::<T>()
			})?
		);

		Ok(())
	}

	#[test]
	fn root_objects() -> Result<()> {
		let mut w = World::new();
		let root = w
			.spawn(T)
			.with_children(|children| {
				children.spawn(T).with_children(|children| {
					children.spawn(T);
					children.spawn(T);
				});
			})
			.id();

		assert!(w.run_system_once(move |objects: Objects<'_, '_, T>| {
			assert_eq!(objects.iter_root().count(), 1);
			assert!(objects.contains_root(root));
			assert!(objects.get_single_root().is_ok());
			true
		})?);

		Ok(())
	}
}
