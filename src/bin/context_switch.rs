mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::{color::palettes::tailwind::FUCHSIA_400, prelude::*};
use bevy_enhanced_input::prelude::*;

use self::player_box::{DEFAULT_SPEED, PlayerBox, PlayerBoxPlugin, PlayerColor};

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
		app.add_input_context::<OnFoot>()
			.add_input_context::<InCar>()
			.add_systems(Startup, spawn)
			.add_observer(apply_movement)
			.add_observer(rotate)
			.add_observer(enter_car)
			.add_observer(exit_car);
	}
}

#[derive(Component)]
struct OnFoot;

impl InputContext for OnFoot {
	fn context_instance(_: &World, _: Entity) -> ContextInstance {
		let mut ctx = ContextInstance::default();

		ctx.bind::<Move>()
			.to(Cardinal::wasd_keys())
			.with_modifiers((
				DeadZone::default(),
				SmoothNudge::default(),
				Scale::splat(DEFAULT_SPEED),
			));

		ctx.bind::<Rotate>().to(KeyCode::Space);
		ctx.bind::<EnterCar>().to(KeyCode::Enter);

		ctx
	}
}

#[derive(Component)]
struct InCar;

impl InputContext for InCar {
	fn context_instance(_: &World, _: Entity) -> ContextInstance {
		let mut ctx = ContextInstance::default();

		ctx.bind::<Move>()
			.to(Cardinal::wasd_keys())
			.with_modifiers((
				DeadZone::default(),
				SmoothNudge::default(),
				Scale::splat(DEFAULT_SPEED + 20.0),
			));

		ctx.bind::<ExitCar>().to(KeyCode::Enter);

		ctx
	}
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct EnterCar;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct ExitCar;

fn exit_car(
	trigger: Trigger<'_, Started<ExitCar>>,
	mut commands: Commands<'_, '_>,
	mut players: Query<'_, '_, &mut PlayerColor>,
) {
	let mut color = players.get_mut(trigger.entity()).unwrap();
	**color = Color::default();

	commands
		.entity(trigger.entity())
		.remove::<InCar>()
		.insert(OnFoot);
}

fn enter_car(
	trigger: Trigger<'_, Started<EnterCar>>,
	mut commands: Commands<'_, '_>,
	mut players: Query<'_, '_, &mut PlayerColor>,
) {
	let mut color = players.get_mut(trigger.entity()).unwrap();
	**color = FUCHSIA_400.into();

	commands
		.entity(trigger.entity())
		.remove::<OnFoot>()
		.insert(InCar);
}

fn rotate(trigger: Trigger<'_, Started<Rotate>>, mut players: Query<'_, '_, &mut Transform>) {
	let mut transform = players.get_mut(trigger.entity()).unwrap();
	transform.rotate_z(FRAC_PI_4);
}

fn apply_movement(trigger: Trigger<'_, Fired<Move>>, mut players: Query<'_, '_, &mut Transform>) {
	let mut transform = players.get_mut(trigger.entity()).unwrap();
	transform.translation += trigger.value.extend(0.0);
}

fn spawn(mut commands: Commands<'_, '_>) {
	commands.spawn(Camera2d);
	commands.spawn((PlayerBox, OnFoot));
}
