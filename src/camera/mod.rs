use bevy::prelude::*;

pub struct CarCameraPlugin;

impl Plugin for CarCameraPlugin {
	fn build(&self, app: &mut App) {}
}

#[derive(Debug, Component)]
pub struct CarCamera;
