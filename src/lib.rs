#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![feature(arbitrary_self_types)]

pub mod debug;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use self::debug::DebugPlugins;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			DefaultPlugins,
			DebugPlugins,
			PhysicsPlugins::default(),
			EnhancedInputPlugin,
		))
		.add_systems(Startup, setup);
	}
}

fn setup(
	mut commands: Commands<'_, '_>,
	mut meshes: ResMut<'_, Assets<Mesh>>,
	mut materials: ResMut<'_, Assets<StandardMaterial>>,
) {
	commands.spawn((
		RigidBody::Static,
		Collider::cylinder(4.0, 0.1),
		Mesh3d(meshes.add(Cylinder::new(4.0, 0.1))),
		MeshMaterial3d(materials.add(Color::WHITE)),
	));

	commands.spawn((
		RigidBody::Dynamic,
		Collider::cuboid(1.0, 1.0, 1.0),
		AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
		Mesh3d(meshes.add(Cuboid::from_length(1.0))),
		MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
		Transform::from_xyz(0.0, 4.0, 0.0),
	));

	commands.spawn((
		PointLight {
			shadows_enabled: true,
			..default()
		},
		Transform::from_xyz(4.0, 8.0, 4.0),
	));

	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Dir3::Y),
	));
}
