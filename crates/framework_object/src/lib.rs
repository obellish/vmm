#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod hierarchy;
mod instance;
mod name;
mod rebind;

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

impl<T: Kind> Clone for Object<'_, '_, '_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Kind> Copy for Object<'_, '_, '_, T> {}

pub struct ObjectRef<'w, 's, 'a, T: Kind = Any>(EntityRef<'a>, Object<'w, 's, 'a, T>);

impl<T: Kind> Clone for ObjectRef<'_, '_, '_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Kind> Copy for ObjectRef<'_, '_, '_, T> {}
