mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use self::player_box::{DEFAULT_SPEED, PlayerBox, PlayerBoxPlugin};

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins,
			EnhancedInputPlugin,
			PlayerBoxPlugin,
			GamePlugin,
		))
		.run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_input_context::<PlayerBox>()
			.add_systems(Startup, spawn)
			.add_observer(apply_movement)
			.add_observer(rotate);
	}
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;

impl InputContext for PlayerBox {
	fn context_instance(_: &World, _: Entity) -> ContextInstance {
		let mut ctx = ContextInstance::default();

		ctx.bind::<Move>()
			.to((Cardinal::wasd_keys(), GamepadStick::Left))
			.with_modifiers((
				DeadZone::default(),
				SmoothNudge::default(),
				Scale::splat(DEFAULT_SPEED),
			));

		ctx.bind::<Rotate>()
			.to((KeyCode::Space, GamepadButton::South));

		ctx
	}
}

fn spawn(mut commands: Commands<'_, '_>) {
	commands.spawn(Camera2d);
	commands.spawn(PlayerBox);
}

fn apply_movement(trigger: Trigger<'_, Fired<Move>>, mut players: Query<'_, '_, &mut Transform>) {
	let mut transform = players.get_mut(trigger.entity()).unwrap();

	transform.translation += trigger.value.extend(0.0);
}

fn rotate(trigger: Trigger<'_, Started<Rotate>>, mut players: Query<'_, '_, &mut Transform>) {
	let mut transform = players.get_mut(trigger.entity()).unwrap();
	transform.rotate_z(FRAC_PI_4);
}
