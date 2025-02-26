#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod debug;

use bevy::prelude::*;

use self::debug::DebugPlugins;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		#[cfg(feature = "debug")]
		app.insert_resource(bevy::dev_tools::picking_debug::DebugPickingMode::Noisy);

		app.add_plugins((DefaultPlugins, DebugPlugins))
			.add_systems(Startup, setup);
	}
}

fn setup(
	mut commands: Commands<'_, '_>,
	mut meshes: ResMut<'_, Assets<Mesh>>,
	mut materials: ResMut<'_, Assets<StandardMaterial>>,
) {
	commands.spawn((
		Mesh3d(meshes.add(Circle::new(4.0))),
		MeshMaterial3d(materials.add(Color::WHITE)),
		Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
	));

	commands.spawn((
		Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
		MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
		Transform::from_xyz(0.0, 0.5, 0.0),
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
		Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
	));
}
