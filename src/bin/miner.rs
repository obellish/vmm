use std::{
	collections::HashMap,
	fmt::{Display, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
	time::Duration,
};

use bevy::{
	color::palettes::css::{AQUAMARINE, GOLD, GREEN_YELLOW, NAVY, ROSY_BROWN, YELLOW_GREEN},
	prelude::*,
	sprite::Anchor,
	time::common_conditions::on_timer,
	window::PrimaryWindow,
};
use bevy_dogoap::prelude::*;
use rand::Rng;

fn main() {
	let mut app = App::new();

	app.add_plugins((
		DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				canvas: Some("#example-canvas".into()),
				..default()
			}),
			..default()
		}),
		DogoapPlugin,
	))
	.add_systems(Startup, setup)
	.add_systems(Update, draw_gizmos)
	.add_systems(
		FixedUpdate,
		(
			print_current_local_state.run_if(on_timer(Duration::from_millis(50))),
			change_hunger_energy_over_time.run_if(on_timer(Duration::from_millis(100))),
			(spawn_random_mushroom, spawn_random_ore).run_if(on_timer(Duration::from_secs(5))),
			(
				handle_go_to_house_action,
				handle_go_to_ore_action,
				handle_go_to_mushroom_action,
				handle_go_to_smelter_action,
				handle_go_to_outside_action,
				handle_go_to_merchant_action,
				handle_eat_action,
				handle_sleep_action,
				handle_mine_ore_action,
				handle_smelt_ore_action,
				handle_sell_metal_action,
			),
		),
	);

	register_components!(
		app,
		vec![Hunger, Energy, AtLocation, HasOre, HasMetal, GoldAmount]
	);

	app.run();
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CurrentAction {
	#[default]
	Idle,
	Sleeping,
	Eating,
	Mining,
	SmeltingOre,
	SellingMetal,
	GoingTo(Location),
}

impl Display for CurrentAction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Idle => "Idle",
			Self::Sleeping => "Sleeping",
			Self::Eating => "Eating",
			Self::Mining => "Mining",
			Self::SmeltingOre => "Smelting ore",
			Self::SellingMetal => "Selling metal",
			Self::GoingTo(Location::House) => "Going to house",
			Self::GoingTo(Location::Outside) => "Going to outside",
			Self::GoingTo(Location::Mushroom) => "Going to mushroom",
			Self::GoingTo(Location::Ore) => "Going to ore",
			Self::GoingTo(Location::Smelter) => "Going to smelter",
			Self::GoingTo(Location::Merchant) => "Going to merchant",
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumDatum, Reflect)]
enum Location {
	#[default]
	House,
	Outside,
	Mushroom,
	Ore,
	Smelter,
	Merchant,
}

#[derive(Component)]
struct Miner;

#[derive(Component)]
struct House;

#[derive(Component)]
struct Smelter;

#[derive(Component)]
struct Mushroom;

#[derive(Component)]
struct Ore;

#[derive(Component)]
struct Merchant;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct EatAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct SleepAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct MineOreAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct SmeltOreAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct SellMetalAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct GoToOutsideAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct GoToHouseAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct GoToMushroomAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct GoToOreAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct GoToSmelterAction;

#[derive(Default, Clone, ActionComponent, Component, Reflect)]
struct GoToMerchantAction;

#[derive(Clone, Component, DatumComponent)]
#[repr(transparent)]
struct Hunger(f64);

impl Deref for Hunger {
	type Target = f64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Hunger {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone, Component, DatumComponent)]
#[repr(transparent)]
struct Energy(f64);

impl Deref for Energy {
	type Target = f64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Energy {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone, Component, EnumComponent)]
#[repr(transparent)]
struct AtLocation(Location);

impl Deref for AtLocation {
	type Target = Location;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for AtLocation {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone, Component, DatumComponent)]
#[repr(transparent)]
struct HasOre(bool);

impl Deref for HasOre {
	type Target = bool;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for HasOre {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone, Component, DatumComponent)]
#[repr(transparent)]
struct HasMetal(bool);

impl Deref for HasMetal {
	type Target = bool;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for HasMetal {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone, Component, DatumComponent)]
#[repr(transparent)]
struct GoldAmount(i64);

impl Deref for GoldAmount {
	type Target = i64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for GoldAmount {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Component)]
struct NeedsText;

fn setup(mut commands: Commands<'_, '_>, windows: Single<'_, &Window, With<PrimaryWindow>>) {
	commands.spawn(Camera2d);

	for i in 0..1 {
		let gold_goal = Goal::from_reqs(&[GoldAmount::is(3)]);

		let sleep_action = SleepAction::new()
			.add_precondition(Energy::is_less(50.0))
			.add_precondition(AtLocation::is(Location::House))
			.add_mutator(Energy::increase(100.0))
			.set_cost(1);

		let eat_action = EatAction::new()
			.add_precondition(Hunger::is_more(50.0))
			.add_precondition(AtLocation::is(Location::Mushroom))
			.add_mutator(Hunger::decrease(25.0))
			.add_mutator(AtLocation::set(Location::Outside))
			.set_cost(2);

		let mine_ore_action = MineOreAction::new()
			.add_precondition(Energy::is_more(10.0))
			.add_precondition(Hunger::is_less(75.0))
			.add_precondition(AtLocation::is(Location::Ore))
			.add_mutator(HasOre::set(true))
			.set_cost(3);

		let smelt_ore_action = SmeltOreAction::new()
			.add_precondition(Energy::is_more(10.0))
			.add_precondition(Hunger::is_less(75.0))
			.add_precondition(AtLocation::is(Location::Smelter))
			.add_precondition(HasOre::is(true))
			.add_mutator(HasOre::set(false))
			.add_mutator(HasMetal::set(true))
			.set_cost(4);

		let sell_metal_action = SellMetalAction::new()
			.add_precondition(Energy::is_more(10.0))
			.add_precondition(Hunger::is_less(75.0))
			.add_precondition(AtLocation::is(Location::Merchant))
			.add_precondition(HasMetal::is(true))
			.add_mutator(GoldAmount::increase(1))
			.add_mutator(HasMetal::set(false))
			.set_cost(5);

		let go_to_outside_action = GoToOutsideAction::new()
			.add_mutator(AtLocation::set(Location::Outside))
			.set_cost(1);

		let go_to_house_action = GoToHouseAction::new()
			.add_precondition(AtLocation::is(Location::Outside))
			.add_mutator(AtLocation::set(Location::House))
			.set_cost(1);

		let go_to_mushroom_action = GoToMushroomAction::new()
			.add_precondition(AtLocation::is(Location::Outside))
			.add_mutator(AtLocation::set(Location::Mushroom))
			.set_cost(2);

		let go_to_ore_action = GoToOreAction::new()
			.add_precondition(AtLocation::is(Location::Outside))
			.add_mutator(AtLocation::set(Location::Ore))
			.set_cost(3);

		let go_to_smelter_action = GoToSmelterAction::new()
			.add_precondition(AtLocation::is(Location::Outside))
			.add_mutator(AtLocation::set(Location::Smelter))
			.set_cost(4);

		let go_to_merchant_action = GoToMerchantAction::new()
			.add_precondition(AtLocation::is(Location::Outside))
			.add_mutator(AtLocation::set(Location::Merchant))
			.set_cost(5);

		let (mut planner, components) = create_planner!({
			actions: [
				(EatAction, eat_action),
				(SleepAction, sleep_action),
				(MineOreAction, mine_ore_action),
				(SmeltOreAction, smelt_ore_action),
				(SellMetalAction, sell_metal_action),
				//
				(GoToOutsideAction, go_to_outside_action),
				(GoToHouseAction, go_to_house_action),
				(GoToMushroomAction, go_to_mushroom_action),
				(GoToOreAction, go_to_ore_action),
				(GoToSmelterAction, go_to_smelter_action),
				(GoToMerchantAction, go_to_merchant_action),
			],
			state: [GoldAmount(0), Hunger(25.0), Energy(75.0), AtLocation(Location::Outside), HasOre(false), HasMetal(false)],
			goals: [gold_goal],
		});

		planner.remove_goal_on_no_plan_found = false;
		planner.always_plan = true;
		planner.current_goal = Some(gold_goal.clone());

		let text_style = TextFont {
			font_size: 18.0,
			..default()
		};

		let transform = Transform::from_translation(Vec3::ZERO.with_x(50.0 * i as f32));

		commands
			.spawn((
				Name::new("miner"),
				Miner,
				planner,
				components,
				transform,
				GlobalTransform::from(transform),
				Visibility::Visible,
			))
			.with_children(|subcommands| {
				subcommands.spawn((
					Transform::from_translation(Vec3::new(10.0, -10.0, 10.0)),
					Text2d(String::new()),
					text_style,
					Anchor::TopLeft,
					NeedsText,
				));
			});

		commands.spawn((
			Name::new("house"),
			House,
			Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
		));

		commands.spawn((
			Name::new("smelter"),
			Smelter,
			Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
		));

		commands.spawn((
			Name::new("merchant"),
			Merchant,
			Transform::from_translation(Vec3::new(-300.0, -50.0, 0.0)),
		));

		let window = &**windows;

		let window_height = window.height() / 2.0;
		let window_width = window.width() / 2.0;

		let mut rng = rand::rng();

		for _ in 0..3 {
			let y = rng.random_range(-window_height..window_height);
			let x = rng.random_range(-window_width..window_width);

			commands.spawn((
				Name::new("mushroom"),
				Mushroom,
				Transform::from_translation(Vec3::new(x, y, 0.0)),
			));
		}

		for _ in 0..10 {
			let y = rng.random_range(-window_height..window_height);
			let x = rng.random_range(-window_width..window_width);

			commands.spawn((
				Name::new("ore"),
				Ore,
				Transform::from_translation(Vec3::new(x, y, 0.0)),
			));
		}
	}
}

fn print_current_local_state(
	query: Query<
		'_,
		'_,
		(
			Entity,
			&Hunger,
			&Energy,
			&HasOre,
			&HasMetal,
			&GoldAmount,
			&Children,
		),
	>,
	q_actions: Query<
		'_,
		'_,
		(
			Option<&SleepAction>,
			Option<&EatAction>,
			Option<&MineOreAction>,
			Option<&SmeltOreAction>,
			Option<&SellMetalAction>,
			Option<&GoToHouseAction>,
			Option<&GoToOutsideAction>,
			Option<&GoToMushroomAction>,
			Option<&GoToOreAction>,
			Option<&GoToSmelterAction>,
			Option<&GoToMerchantAction>,
		),
	>,
	q_child: Query<'_, '_, Entity, With<NeedsText>>,
	mut text_writer: Text2dWriter<'_, '_>,
) {
	for (
		entity,
		&Hunger(hunger),
		&Energy(energy),
		&HasOre(has_ore),
		&HasMetal(has_metal),
		&GoldAmount(gold_amount),
		children,
	) in &query
	{
		let mut current_action = CurrentAction::Idle;

		let (
			sleep,
			eat,
			mine,
			smelting,
			selling_metal,
			go_to_house,
			go_to_outside,
			go_to_mushroom,
			go_to_ore,
			go_to_smelter,
			go_to_merchant,
		) = q_actions.get(entity).unwrap();

		if sleep.is_some() {
			current_action = CurrentAction::Sleeping;
		}

		if eat.is_some() {
			current_action = CurrentAction::Eating;
		}

		if mine.is_some() {
			current_action = CurrentAction::Mining;
		}

		if smelting.is_some() {
			current_action = CurrentAction::SmeltingOre;
		}

		if selling_metal.is_some() {
			current_action = CurrentAction::SellingMetal;
		}

		if go_to_house.is_some() {
			current_action = CurrentAction::GoingTo(Location::House);
		}

		if go_to_outside.is_some() {
			current_action = CurrentAction::GoingTo(Location::Outside);
		}

		if go_to_mushroom.is_some() {
			current_action = CurrentAction::GoingTo(Location::Mushroom);
		}

		if go_to_ore.is_some() {
			current_action = CurrentAction::GoingTo(Location::Ore);
		}

		if go_to_smelter.is_some() {
			current_action = CurrentAction::GoingTo(Location::Smelter);
		}

		if go_to_merchant.is_some() {
			current_action = CurrentAction::GoingTo(Location::Merchant);
		}

		for &child in children {
			let text = q_child.get(child).unwrap();
			*text_writer.text(text, 0) = format!(
				"{current_action}\nGold: {gold_amount}\nHunger: {hunger:.0}\nEnergy: {energy:.0}\nHas Ore? {has_ore}\nHas Metal? {has_metal}"
			);
		}
	}
}

fn draw_gizmos(
	mut gizmos: Gizmos<'_, '_>,
	q_miner: Query<'_, '_, &Transform, With<Miner>>,
	q_house: Query<'_, '_, &Transform, With<House>>,
	q_smelter: Query<'_, '_, &Transform, With<Smelter>>,
	q_merchant: Query<'_, '_, &Transform, With<Merchant>>,
	q_mushrooms: Query<'_, '_, &Transform, With<Mushroom>>,
	q_ore: Query<'_, '_, &Transform, With<Ore>>,
) {
	gizmos
		.grid_2d(
			Vec2::ZERO,
			UVec2::new(16, 9),
			Vec2::splat(80.0),
			Srgba::new(0.1, 0.1, 0.1, 0.5),
		)
		.outer_edges();

	for miner in &q_miner {
		gizmos.circle_2d(miner.translation.truncate(), 16.0, NAVY);
	}

	let Ok(house) = q_house.get_single() else {
		return;
	};

	gizmos.rect_2d(
		house.translation.truncate(),
		Vec2::new(40.0, 80.0),
		AQUAMARINE,
	);

	let Ok(smelter) = q_smelter.get_single() else {
		return;
	};

	gizmos.rect_2d(
		smelter.translation.truncate(),
		Vec2::splat(30.0),
		YELLOW_GREEN,
	);

	let Ok(merchant) = q_merchant.get_single() else {
		return;
	};

	gizmos.circle_2d(merchant.translation.truncate(), 16.0, GOLD);

	for mushroom in &q_mushrooms {
		gizmos.circle_2d(mushroom.translation.truncate(), 4.0, GREEN_YELLOW);
	}

	for ore in &q_ore {
		gizmos.circle_2d(ore.translation.truncate(), 4.0, ROSY_BROWN);
	}
}

fn change_hunger_energy_over_time(
	time: Res<'_, Time>,
	mut query: Query<'_, '_, (&mut Hunger, &mut Energy)>,
) {
	let mut rng = rand::rng();
	for (mut hunger, mut energy) in &mut query {
		let r = rng.random_range(10.0..20.0);
		let value = r * time.delta_secs_f64();
		**hunger += value;

		if **hunger > 100.0 {
			**hunger = 100.0;
		}

		let r = rng.random_range(1.0..10.0);
		let value = r * time.delta_secs_f64();
		**energy -= value;
		if **energy < 0.0 {
			**energy = 0.0;
		}
	}
}

fn spawn_random_mushroom(
	windows: Single<'_, &Window, With<PrimaryWindow>>,
	mut commands: Commands<'_, '_>,
	q_mushrooms: Query<'_, '_, Entity, With<Mushroom>>,
) {
	if q_mushrooms.iter().len() < 10 {
		let window = &**windows;
		let window_height = window.height() / 2.0;
		let window_width = window.width() / 2.0;

		let mut rng = rand::rng();
		let y = rng.random_range(-window_height..window_height);
		let x = rng.random_range(-window_width..window_width);

		commands.spawn((
			Name::new("mushroom"),
			Mushroom,
			Transform::from_translation(Vec3::new(x, y, 0.0)),
		));
	}
}

fn spawn_random_ore(
	windows: Single<'_, &Window, With<PrimaryWindow>>,
	mut commands: Commands<'_, '_>,
	q_ores: Query<'_, '_, Entity, With<Ore>>,
) {
	if q_ores.iter().len() < 10 {
		let window = &**windows;
		let window_height = window.height() / 2.0;
		let window_width = window.width() / 2.0;

		let mut rng = rand::rng();
		let y = rng.random_range(-window_height..window_height);
		let x = rng.random_range(-window_width..window_width);

		commands.spawn((
			Name::new("ore"),
			Ore,
			Transform::from_translation(Vec3::new(x, y, 0.0)),
		));
	}
}

fn handle_go_to_house_action(
	mut commands: Commands<'_, '_>,
	time: Res<'_, Time>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut AtLocation),
		(Without<House>, With<GoToHouseAction>),
	>,
	q_house: Query<'_, '_, &Transform, With<House>>,
) {
	for (entity, mut t_entity, mut at_location) in &mut query {
		let Ok(t_house) = q_house.get_single() else {
			return;
		};

		go_to_location::<GoToHouseAction>(
			&mut at_location,
			time.delta_secs(),
			&mut t_entity,
			t_house.translation,
			Location::House,
			entity,
			&mut commands,
		);
	}
}

fn handle_go_to_ore_action(
	mut commands: Commands<'_, '_>,
	time: Res<'_, Time>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut AtLocation),
		(Without<Ore>, With<GoToOreAction>),
	>,
	q_world_resource: Query<'_, '_, (Entity, &Transform), With<Ore>>,
) {
	for (entity, mut t_entity, mut at_location) in &mut query {
		let origin = t_entity.translation;
		let items = q_world_resource
			.iter()
			.map(|(e, t)| (e, *t))
			.collect::<Vec<_>>();
		let closest = find_closest(origin, &items).unwrap();

		go_to_location::<GoToOreAction>(
			&mut at_location,
			time.delta_secs(),
			&mut t_entity,
			closest.1,
			Location::Ore,
			entity,
			&mut commands,
		);
	}
}

fn handle_go_to_mushroom_action(
	mut commands: Commands<'_, '_>,
	time: Res<'_, Time>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut AtLocation),
		(Without<Mushroom>, With<GoToMushroomAction>),
	>,
	q_mushrooms: Query<'_, '_, (Entity, &Transform), With<Mushroom>>,
) {
	for (entity, mut t_entity, mut at_location) in &mut query {
		let origin = t_entity.translation;
		let items = q_mushrooms.iter().map(|(e, t)| (e, *t)).collect::<Vec<_>>();
		let mushroom = find_closest(origin, &items).unwrap();

		go_to_location::<GoToMushroomAction>(
			&mut at_location,
			time.delta_secs(),
			&mut t_entity,
			mushroom.1,
			Location::Mushroom,
			entity,
			&mut commands,
		);
	}
}

fn handle_go_to_smelter_action(
	mut commands: Commands<'_, '_>,
	time: Res<'_, Time>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut AtLocation),
		(With<GoToSmelterAction>, Without<Smelter>),
	>,
	q_smelter: Query<'_, '_, &Transform, With<Smelter>>,
) {
	for (entity, mut t_entity, mut at_location) in &mut query {
		let Ok(t_smelter) = q_smelter.get_single() else {
			return;
		};

		go_to_location::<GoToSmelterAction>(
			&mut at_location,
			time.delta_secs(),
			&mut t_entity,
			t_smelter.translation,
			Location::Smelter,
			entity,
			&mut commands,
		);
	}
}

fn handle_go_to_outside_action(
	mut commands: Commands<'_, '_>,
	time: Res<'_, Time>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut AtLocation),
		(Without<House>, With<GoToOutsideAction>),
	>,
	q_house: Query<'_, '_, &Transform, With<House>>,
) {
	for (entity, mut t_entity, mut at_location) in &mut query {
		let Ok(t_house) = q_house.get_single() else {
			return;
		};

		let offset = Vec3::new(-30.0, 0.0, 0.0);
		let new_pos = t_house.translation + offset;

		go_to_location::<GoToOutsideAction>(
			&mut at_location,
			time.delta_secs(),
			&mut t_entity,
			new_pos,
			Location::Outside,
			entity,
			&mut commands,
		);
	}
}

fn handle_go_to_merchant_action(
	mut commands: Commands<'_, '_>,
	time: Res<'_, Time>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut AtLocation),
		(Without<Merchant>, With<GoToMerchantAction>),
	>,
	q_destination: Query<'_, '_, &Transform, With<Merchant>>,
) {
	for (entity, mut t_entity, mut at_location) in &mut query {
		let Ok(t_destination) = q_destination.get_single() else {
			return;
		};

		go_to_location::<GoToMerchantAction>(
			&mut at_location,
			time.delta_secs(),
			&mut t_entity,
			t_destination.translation,
			Location::Merchant,
			entity,
			&mut commands,
		);
	}
}

fn handle_eat_action(
	mut commands: Commands<'_, '_>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut Transform, &mut Hunger, &mut AtLocation),
		(Without<Mushroom>, With<EatAction>),
	>,
	q_mushrooms: Query<'_, '_, (Entity, &Transform), With<Mushroom>>,
) {
	for (entity, t_entity, mut hunger, mut at_location) in &mut query {
		let origin = t_entity.translation;
		let items = q_mushrooms.iter().map(|(e, t)| (e, *t)).collect::<Vec<_>>();
		let mushroom = find_closest(origin, &items);

		info!("Eating mushroom we found at {mushroom:?}");

		let mushroom = mushroom.unwrap();

		**hunger -= 50.0;

		commands.entity(entity).remove::<EatAction>();
		commands.entity(mushroom.0).despawn_recursive();

		**at_location = Location::Outside;
	}
}

fn handle_sleep_action(
	time: Res<'_, Time>,
	mut commands: Commands<'_, '_>,
	mut query: Query<'_, '_, (Entity, &mut Energy, &mut Planner), With<SleepAction>>,
) {
	let mut rng = rand::rng();
	for (entity, mut energy, mut planner) in &mut query {
		planner.always_plan = false;

		let r = rng.random_range(5.0..20.0);
		let value = r * time.delta_secs_f64();
		**energy += value;
		if **energy >= 100.0 {
			commands.entity(entity).remove::<SleepAction>();

			commands.entity(entity).insert(GoToOutsideAction);
			**energy = 100.0;

			planner.always_plan = true;
		}
	}
}

fn handle_mine_ore_action(
	time: Res<'_, Time>,
	mut commands: Commands<'_, '_>,
	mut query: Query<
		'_,
		'_,
		(
			Entity,
			&mut Transform,
			&mut HasOre,
			&mut AtLocation,
			&mut Energy,
		),
		(With<MineOreAction>, Without<Ore>),
	>,
	q_ores: Query<'_, '_, (Entity, &Transform), With<Ore>>,
	mut mining_progress: Local<'_, HashMap<Entity, Timer>>,
) {
	for (entity, t_entity, mut has_ore, mut at_location, mut energy) in &mut query {
		let origin = t_entity.translation;
		let items = q_ores.iter().map(|(e, t)| (e, *t)).collect::<Vec<_>>();
		let closest = find_closest(origin, &items).unwrap();

		action_with_progress(
			&mut mining_progress,
			closest.0,
			&time,
			2.0,
			|is_completed| {
				if is_completed {
					**has_ore = true;
					**at_location = Location::Outside;

					commands.entity(entity).remove::<MineOreAction>();
					commands.entity(closest.0).despawn_recursive();
				} else {
					let mut rng = rand::rng();

					let r = rng.random_range(5.0..10.0);
					let value = r * time.delta_secs_f64();
					**energy -= value;
					if **energy <= 0.0 {
						commands.entity(entity).remove::<MineOreAction>();
						**energy = 0.0;
					}
				}
			},
		);
	}
}

fn handle_smelt_ore_action(
	time: Res<'_, Time>,
	mut commands: Commands<'_, '_>,
	mut query: Query<
		'_,
		'_,
		(
			Entity,
			&mut Transform,
			&mut Energy,
			&mut HasOre,
			&mut HasMetal,
			&mut AtLocation,
		),
		(Without<Smelter>, With<SmeltOreAction>),
	>,
	q_smelters: Query<'_, '_, (Entity, &Transform), With<Smelter>>,
	mut progress: Local<'_, HashMap<Entity, Timer>>,
) {
	for (entity, t_entity, mut energy, mut has_ore, mut has_metal, mut at_location) in &mut query {
		let origin = t_entity.translation;
		let items = q_smelters.iter().map(|(e, t)| (e, *t)).collect::<Vec<_>>();
		let closest = find_closest(origin, &items).unwrap();

		action_with_progress(&mut progress, closest.0, &time, 5.0, |is_completed| {
			if is_completed {
				**has_metal = true;
				**has_ore = false;
				**at_location = Location::Outside;

				commands.entity(entity).remove::<SmeltOreAction>();
			} else {
				let mut rng = rand::rng();
				let r = rng.random_range(10.0..15.0);
				let value = r * time.delta_secs_f64();
				**energy -= value;
				if **energy <= 0.0 {
					commands.entity(entity).remove::<SmeltOreAction>();
					**energy = 0.0;
				}
			}
		});
	}
}

fn handle_sell_metal_action(
	time: Res<'_, Time>,
	mut commands: Commands<'_, '_>,
	mut query: Query<
		'_,
		'_,
		(Entity, &mut HasMetal, &mut GoldAmount, &mut AtLocation),
		(With<SellMetalAction>, Without<Smelter>),
	>,
	mut progress: Local<'_, HashMap<Entity, Timer>>,
) {
	for (entity, mut has_metal, mut gold_amount, mut at_location) in &mut query {
		action_with_progress(&mut progress, entity, &time, 1.0, |is_completed| {
			if is_completed {
				**has_metal = false;

				**gold_amount += 1;

				**at_location = Location::Outside;

				commands.entity(entity).remove::<SellMetalAction>();
			}
		});
	}
}

fn go_to_location<T>(
	at_location: &mut AtLocation,
	delta: f32,
	origin: &mut Transform,
	destination: Vec3,
	destination_enum: Location,
	entity: Entity,
	commands: &mut Commands<'_, '_>,
) where
	T: Component,
{
	if origin.translation.distance(destination) > 5.0 {
		let direction = (destination - origin.translation).normalize();
		origin.translation += direction * 128.0 * delta;
	} else {
		**at_location = destination_enum;
		commands.entity(entity).remove::<T>();
	}
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn action_with_progress(
	progresses: &mut HashMap<Entity, Timer>,
	entity: Entity,
	time: &Time,
	delay_seconds: f32,
	on_progress: impl FnOnce(bool),
) {
	let progress = progresses.get_mut(&entity);

	match progress {
		Some(progress) => {
			if progress.tick(time.delta()).just_finished() {
				on_progress(true);
				progresses.remove(&entity);
			} else {
				on_progress(false);
			}
		}
		None => {
			progresses.insert(entity, Timer::from_seconds(delay_seconds, TimerMode::Once));
		}
	}
}

fn find_closest<'a>(
	origin: Vec3,
	items: impl IntoIterator<Item = &'a (Entity, Transform)>,
) -> Option<(Entity, Vec3)> {
	let mut closest: Option<(Entity, Transform, f32)> = None;
	for (entity, transform) in items {
		match closest {
			Some((.., d)) => {
				let distance = transform.translation.distance(origin);
				if distance < d {
					closest = Some((*entity, *transform, distance));
				}
			}
			None => closest = Some((*entity, *transform, 1000.0)),
		}
	}

	closest.map(|(e, t, _)| (e, t.translation))
}
