use bevy_ecs::{
	prelude::*,
	query::{QueryData, QueryFilter, QueryItem},
};
use bevy_framework_kind::{Any, prelude::*};

use super::{Object, ObjectInstance, ObjectName, ObjectRebind, ObjectRef, Objects};

pub trait ObjectHierarchy<T: Kind = Any>: ObjectRebind<T> + ObjectName {
	fn parent(&self) -> Option<Self::Rebind<Any>>;

	fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

	fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

	fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

	fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

	fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>>;

	fn root(&self) -> Self::Rebind<Any> {
		self.ancestors()
			.last()
			.unwrap_or_else(|| self.rebind_any(self.entity()))
	}

	fn is_root(&self) -> bool {
		self.parent().is_none()
	}

	fn is_child(&self) -> bool {
		!self.is_root()
	}

	fn is_child_of(&self, parent: Entity) -> bool {
		self.parent()
			.is_some_and(|object| object.entity() == parent)
	}

	fn has_children(&self) -> bool {
		self.children().next().is_some()
	}

	fn query_children<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.children()
			.filter_map(move |object| query.get(object.entity()).ok())
	}

	fn children_of_kind<'a, U: Kind>(
		&'a self,
		objects: &'a Objects<'_, '_, U>,
	) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
		self.children()
			.filter_map(move |object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(ObjectInstance::instance(&object)))
	}

	fn find_children_of_kind<U: Kind>(
		&self,
		objects: &Objects<'_, '_, U>,
	) -> Option<Self::Rebind<U>> {
		self.children()
			.find_map(|object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}

	fn self_and_ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		std::iter::once(self.rebind_any(self.entity())).chain(self.ancestors())
	}

	fn is_ancestor_of(&self, entity: Entity) -> bool
	where
		Self::Rebind<Any>: ObjectHierarchy<Any>,
	{
		self.rebind_any(entity)
			.ancestors()
			.any(|ancestor| ancestor.entity() == self.entity())
	}

	fn query_ancestors<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.ancestors().filter_map(move |object| {
			let entity = object.entity();
			query.get(entity).ok()
		})
	}

	fn query_self_and_ancestors<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.self_and_ancestors().filter_map(move |object| {
			let entity = object.entity();
			query.get(entity).ok()
		})
	}

	fn ancestors_of_kind<'a, U: Kind>(
		&'a self,
		objects: &'a Objects<'_, '_, U>,
	) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
		self.ancestors()
			.filter_map(move |object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}

	fn find_ancestor_of_kind<U: Kind>(
		&self,
		objects: &Objects<'_, '_, U>,
	) -> Option<Self::Rebind<U>> {
		self.ancestors()
			.find_map(|object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}

	fn self_and_descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		std::iter::once(self.rebind_any(self.entity())).chain(self.descendants_wide())
	}

	fn self_and_descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		std::iter::once(self.rebind_any(self.entity())).chain(self.descendants_deep())
	}

	fn is_descendant_of(&self, entity: Entity) -> bool
	where
		Self::Rebind<Any>: ObjectHierarchy<Any>,
	{
		self.ancestors().any(|ancestor| ancestor.entity() == entity)
	}

	fn descendants_of_kind_wide<'a, U: Kind>(
		&'a self,
		objects: &'a Objects<'_, '_, U>,
	) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
		self.descendants_wide()
			.filter_map(move |object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}

	fn descendants_of_kind_deep<'a, U: Kind>(
		&'a self,
		objects: &'a Objects<'_, '_, U>,
	) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
		self.descendants_deep()
			.filter_map(move |object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}

	fn query_descendants_wide<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.descendants_wide().filter_map(move |object| {
			let entity = object.entity();
			query.get(entity).ok()
		})
	}

	fn query_descendants_deep<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.descendants_deep().filter_map(move |object| {
			let entity = object.entity();
			query.get(entity).ok()
		})
	}

	fn query_self_and_descendants_wide<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.self_and_descendants_wide().filter_map(move |object| {
			let entity = object.entity();
			query.get(entity).ok()
		})
	}

	fn query_self_and_descendants_deep<'a, Q: QueryData, F: QueryFilter>(
		&'a self,
		query: &'a Query<'_, '_, Q, F>,
	) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
		self.self_and_descendants_deep().filter_map(move |object| {
			let entity = object.entity();
			query.get(entity).ok()
		})
	}

	fn find_descendant_of_kind_wide<U: Kind>(
		&self,
		objects: &Objects<'_, '_, U>,
	) -> Option<Self::Rebind<U>> {
		self.descendants_wide()
			.find_map(|object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}

	fn find_descendant_of_kind_deep<U: Kind>(
		&self,
		objects: &Objects<'_, '_, U>,
	) -> Option<Self::Rebind<U>> {
		self.descendants_deep()
			.find_map(|object| objects.get(object.entity()).ok())
			.map(|object| self.rebind_as(object.instance()))
	}
}

impl<T: Kind> ObjectHierarchy<T> for Object<'_, '_, '_, T> {
	fn parent(&self) -> Option<Self::Rebind<Any>> {
		self.hierarchy
			.parent(self.entity())
			.map(|entity| self.rebind_any(entity))
	}

	fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.hierarchy
			.children(self.entity())
			.map(|entity| self.rebind_any(entity))
	}

	fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.hierarchy
			.ancestors(self.entity())
			.map(|entity| self.rebind_any(entity))
	}

	fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.hierarchy
			.descendants_wide(self.entity())
			.map(|entity| self.rebind_any(entity))
	}

	fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.hierarchy
			.descendants_deep(self.entity())
			.map(|entity| self.rebind_any(entity))
	}

	fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>> {
		let tail = path.as_ref().split('/').collect::<Vec<_>>();
		find_by_path(self.cast_into_any(), &tail)
	}
}

impl<T: Kind> ObjectHierarchy<T> for ObjectRef<'_, '_, '_, T> {
	fn parent(&self) -> Option<Self::Rebind<Any>> {
		self.1.parent().map(|object| ObjectRef(self.0, object))
	}

	fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.1.children().map(|object| ObjectRef(self.0, object))
	}

	fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.1.ancestors().map(|object| ObjectRef(self.0, object))
	}

	fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.1
			.descendants_wide()
			.map(|object| ObjectRef(self.0, object))
	}

	fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
		self.1
			.descendants_deep()
			.map(|object| ObjectRef(self.0, object))
	}

	fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>> {
		self.1
			.find_by_path(path)
			.map(|object| ObjectRef(self.0, object))
	}
}

fn find_by_path<T: ObjectHierarchy<Rebind<Any> = T>>(
	curr: T,
	tail: &[&str],
) -> Option<T::Rebind<Any>> {
	if tail.is_empty() {
		return Some(curr);
	}

	let head = tail[0];
	let tail = &tail[1..];

	if matches!(head, ".") || head.is_empty() {
		find_by_path(curr, tail)
	} else if matches!(head, "..") {
		curr.parent()
			.map_or_else(|| None, |parent| find_by_path(parent, tail))
	} else if matches!(head, "*") {
		for child in curr.children() {
			if let Some(result) = find_by_path(child, tail) {
				return Some(result);
			}
		}

		return None;
	} else if let Some(child) = curr
		.children()
		.find(|part| part.name().is_some_and(|name| name == head))
	{
		find_by_path(child, tail)
	} else {
		None
	}
}
