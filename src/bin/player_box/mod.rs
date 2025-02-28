use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

pub(super) const DEFAULT_SPEED: f32 = 10.0;

pub(super) struct PlayerBoxPlugin;

impl Plugin for PlayerBoxPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostUpdate, update_position);
	}
}

#[derive(Default, Component)]
#[require(PlayerColor, Visibility, Transform)]
#[repr(transparent)]
pub(super) struct PlayerBox;

#[derive(Default, Component)]
#[repr(transparent)]
pub(super) struct PlayerColor(pub(super) Color);

impl Deref for PlayerColor {
	type Target = Color;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for PlayerColor {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

fn update_position(
	mut gizmos: Gizmos<'_, '_>,
	players: Query<'_, '_, (&Visibility, &Transform, &PlayerColor)>,
) {
	for (visibility, transform, color) in &players {
		if !matches!(visibility, Visibility::Hidden) {
			const DEFAULT_SCALE: Vec2 = Vec2::splat(50.0);
			gizmos.rect(
				Isometry3d::new(transform.translation, transform.rotation),
				DEFAULT_SCALE + transform.scale.xy(),
				color.0,
			);
		}
	}
}
