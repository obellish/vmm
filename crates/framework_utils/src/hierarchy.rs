use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_hierarchy::prelude::*;

#[derive(SystemParam)]
pub struct HierarchyQuery<'w, 's> {
	parent: Query<'w, 's, &'static Parent>,
	children: Query<'w, 's, &'static Children>,
}

impl HierarchyQuery<'_, '_> {
	#[must_use]
	pub fn parent(&self, entity: Entity) -> Option<Entity> {
		self.parent.get(entity).ok().map(|parent| **parent)
	}

	pub fn children(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.children
			.get(entity)
			.ok()
			.into_iter()
			.flat_map(|children| children.into_iter().copied())
	}

	pub fn ancestors(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.parent.iter_ancestors(entity)
	}

	pub fn descendants_wide(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.children.iter_descendants(entity)
	}

	pub fn descendants_deep(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
		self.children.iter_descendants_depth_first(entity)
	}
}
