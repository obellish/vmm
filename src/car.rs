use bevy::{color::palettes::css::ORANGE, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_rapier3d::prelude::*;
use vmm_utils::WithTransform;

#[derive(Debug, Component)]
pub struct Car;

impl Car {
	pub fn spawn(
		mut commands: Commands<'_, '_>,
		mut meshes: ResMut<'_, Assets<Mesh>>,
		mut materials: ResMut<'_, Assets<StandardMaterial>>,
	) {
		let car_id = commands
			.spawn((
				Self,
				Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 1.0))),
				MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
				(
					RigidBody::Dynamic,
					Velocity::default(),
					Collider::cuboid(1.0, 1.0, 1.0),
					ColliderDebugColor(ORANGE.into()),
				),
			))
			.id();

		commands.entity(car_id).with_children(|parent| {
			parent.spawn(
				Wheel::spawn()
					.with_transform(Transform::from_translation(Vec3::new(-1.5, -0.5, 0.0))),
			);

			parent.spawn(
				Wheel::spawn()
					.with_transform(Transform::from_translation(Vec3::new(1.5, -0.5, 0.0))),
			);
		});
	}
}

#[derive(Debug, Component)]
pub struct Wheel;

impl Wheel {
	fn spawn() -> impl Bundle {
		(
			Self,
			ActiveEvents::COLLISION_EVENTS,
			Collider::ball(1.0),
			ColliderDebugColor(ORANGE.into()),
			Friction::coefficient(0.1),
			Restitution::coefficient(0.0),
		)
	}
}
