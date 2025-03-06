use bevy_ecs::prelude::*;

#[must_use]
pub const fn has_resource<T: Resource>(resource: Option<Res<'_, T>>) -> bool {
	resource.is_some()
}

#[must_use]
pub fn has_event<T: Event>(events: EventReader<'_, '_, T>) -> bool {
	!events.is_empty()
}

pub fn remove_resource<T: Resource>(mut commands: Commands<'_, '_>) {
	commands.remove_resource::<T>();
}

pub fn remove_resource_immediate<T: Resource>(world: &mut World) {
	world.remove_resource::<T>();
}
