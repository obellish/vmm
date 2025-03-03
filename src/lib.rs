#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![feature(arbitrary_self_types)]
#![allow(unused)]

pub mod camera;
pub mod car;
#[cfg(feature = "debug")]
pub mod debug;

use bevy::{
	prelude::*,
	render::mesh::{SphereKind, SphereMeshBuilder},
};
use bevy_enhanced_input::prelude::*;
use bevy_rapier3d::prelude::*;
use vmm_utils::prelude::*;

#[cfg(feature = "debug")]
use self::debug::DebugPlugins;
use self::{
	camera::{CarCamera, CarCameraPlugin},
	car::Car,
};

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		#[cfg(feature = "debug")]
		app.add_plugins(DebugPlugins);

		app.add_plugins((
			DefaultPlugins,
			RapierPhysicsPlugin::<NoUserData>::default(),
			RapierPickingPlugin,
			EnhancedInputPlugin,
			CarCameraPlugin,
		))
		.add_systems(Startup, (setup, Car::spawn));
	}
}

fn setup(mut commands: Commands<'_, '_>, asset_server: Res<'_, AssetServer>) {
	commands.spawn(
		PointLight {
			shadows_enabled: true,
			..default()
		}
		.with_xyz(0.0, 4.0, 0.0),
	);

	commands.spawn(Collider::cuboid(100.0, 0.1, 100.0).with_xyz(0.0, -2.0, 0.0));

	commands.spawn(
		CarCamera
			.with_transform(Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Dir3::Y)),
	);
}
