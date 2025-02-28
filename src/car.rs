use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Debug, Component)]
pub struct Car;

impl InputContext for Car {
	fn context_instance(_: &World, _: Entity) -> ContextInstance {
		let mut ctx = ContextInstance::default();

		ctx
	}
}
