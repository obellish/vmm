#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![feature(arbitrary_self_types)]
#![allow(unused)]

pub mod camera;
#[cfg(feature = "debug")]
pub mod debug;

use bevy::{
	log::LogPlugin,
	prelude::*,
	render::mesh::{SphereKind, SphereMeshBuilder},
};
use bevy_enhanced_input::prelude::*;
use vmm_utils::prelude::*;

use self::camera::PlayerCamera;
#[cfg(feature = "debug")]
use self::debug::DebugPlugins;

pub struct MainPlugin;

impl Plugin for MainPlugin {
	fn build(&self, app: &mut App) {
		#[cfg(feature = "debug")]
		app.add_plugins(DebugPlugins);

		app.add_plugins((
			DefaultPlugins.set(LogPlugin {
				filter: "debug,wgpu_core=error,wgpu_hal=error,naga=error,vmm=debug".to_owned(),
				level: bevy::log::Level::DEBUG,
				..default()
			}),
			EnhancedInputPlugin,
		))
		.add_systems(Startup, setup);
	}
}

fn setup(mut commands: Commands<'_, '_>) {
	commands.spawn(
		PointLight {
			shadows_enabled: true,
			..default()
		}
		.with_xyz(0.0, 4.0, 0.0),
	);

	commands.spawn(
		PlayerCamera
			.with_transform(Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Dir3::Y)),
	);
}
