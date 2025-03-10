use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{self, Read},
	path::PathBuf,
};

use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::{entity::EntityHashMap, prelude::*, query::QueryFilter, schedule::SystemConfigs};
use bevy_framework_utils::prelude::*;
use bevy_hierarchy::DespawnRecursiveExt as _;
use bevy_scene::{SceneSpawnError, serde::SceneDeserializer};
use bevy_utils::tracing::{error, info, warn};
use serde::de::DeserializeSeed;

use super::{
	FileFromEvent, FileFromResource, GetFilePath, GetStaticStream, GetStream, MapComponent,
	Pipeline, SceneMapper, StaticFile, StaticStream, StreamFromEvent, StreamFromResource,
	save::{Save, SaveSet, Saved},
};

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
	fn build(&self, app: &mut App) {
		app.configure_sets(
			PreUpdate,
			(
				LoadSet::Load,
				LoadSet::PostLoad.run_if(has_resource::<Loaded>),
			)
				.chain()
				.before(SaveSet::Save),
		)
		.add_systems(
			PreUpdate,
			remove_resource::<Loaded>.in_set(LoadSet::PostLoad),
		);
	}
}

#[derive(Default, Clone, Component)]
pub struct Unload;

#[derive(Resource)]
#[repr(transparent)]
pub struct Loaded {
	pub entity_map: EntityHashMap<Entity>,
}

pub struct LoadPipelineBuilder<P> {
	pipeline: P,
	mapper: SceneMapper,
}

impl<P> LoadPipelineBuilder<P> {
	#[must_use]
	pub fn map_component<U: Component>(self, m: impl MapComponent<U>) -> Self {
		Self {
			mapper: self.mapper.map(m),
			..self
		}
	}
}

impl<P: LoadPipeline> LoadPipeline for LoadPipelineBuilder<P> {
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		let mapper = self.mapper.clone();
		IntoSystem::into_system(self.pipeline.load().pipe(
			move |In(saved): In<Result<Saved, LoadError>>| {
				saved.map(|saved| Saved {
					mapper: mapper.clone(),
					..saved
				})
			},
		))
	}
}

impl<P: Pipeline> Pipeline for LoadPipelineBuilder<P> {
	fn finish(&self, pipeline: impl System<In = (), Out = ()>) -> SystemConfigs {
		self.pipeline.finish(pipeline)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum LoadSet {
	Load,
	PostLoad,
}

#[derive(Debug)]
pub enum LoadError {
	Io(io::Error),
	De(ron::de::SpannedError),
	Ron(ron::Error),
	Scene(SceneSpawnError),
}

impl Display for LoadError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(i) => Display::fmt(&i, f),
			Self::De(d) => Display::fmt(&d, f),
			Self::Ron(r) => Display::fmt(&r, f),
			Self::Scene(s) => Display::fmt(&s, f),
		}
	}
}

impl StdError for LoadError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(i) => Some(i),
			Self::De(d) => Some(d),
			Self::Ron(r) => Some(r),
			Self::Scene(s) => Some(s),
		}
	}
}

impl From<io::Error> for LoadError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<ron::de::SpannedError> for LoadError {
	fn from(value: ron::de::SpannedError) -> Self {
		Self::De(value)
	}
}

impl From<ron::Error> for LoadError {
	fn from(value: ron::Error) -> Self {
		Self::Ron(value)
	}
}

impl From<SceneSpawnError> for LoadError {
	fn from(value: SceneSpawnError) -> Self {
		Self::Scene(value)
	}
}

pub trait LoadPipeline: Pipeline {
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>>;
}

impl LoadPipeline for StaticFile {
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		IntoSystem::into_system(read_static_file(self.0.clone(), SceneMapper::default()))
	}
}

impl<S: GetStaticStream> LoadPipeline for StaticStream<S>
where
	S::Stream: Read,
{
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		IntoSystem::into_system((|| S::stream()).pipe(read_stream))
	}
}

impl<R> LoadPipeline for FileFromResource<R>
where
	R: GetFilePath + Resource,
{
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		IntoSystem::into_system(get_file_from_resource::<R>.pipe(read_file))
	}
}

impl<R> LoadPipeline for StreamFromResource<R>
where
	R: GetStream + Resource,
	R::Stream: Read,
{
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		IntoSystem::into_system((|resource: Res<'_, R>| resource.stream()).pipe(read_stream))
	}
}

impl<E> LoadPipeline for FileFromEvent<E>
where
	E: Event + GetFilePath,
{
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		IntoSystem::into_system(get_file_from_event::<E>.pipe(read_file))
	}
}

impl<E> LoadPipeline for StreamFromEvent<E>
where
	E: Event + GetStream,
	E::Stream: Read,
{
	fn load(&self) -> impl System<In = (), Out = Result<Saved, LoadError>> {
		IntoSystem::into_system(get_stream_from_event::<E>.pipe(read_stream))
	}
}

pub trait LoadMapComponent: Sized {
	fn map_component<U: Component>(self, m: impl MapComponent<U>) -> LoadPipelineBuilder<Self>;
}

impl<P: Pipeline> LoadMapComponent for P {
	fn map_component<U: Component>(self, m: impl MapComponent<U>) -> LoadPipelineBuilder<Self> {
		LoadPipelineBuilder {
			pipeline: self,
			mapper: SceneMapper::default().map(m),
		}
	}
}

pub type DefaultUnloadFilter = Or<(With<Save>, With<Unload>)>;

pub fn load(p: impl LoadPipeline) -> SystemConfigs {
	let system = p
		.load()
		.pipe(unload::<DefaultUnloadFilter>)
		.pipe(write_to_world)
		.pipe(insert_into_loaded(Save))
		.pipe(insert_loaded);
	p.finish(IntoSystem::into_system(system))
		.in_set(LoadSet::Load)
}

pub fn read_static_file(
	path: impl Into<PathBuf>,
	mapper: SceneMapper,
) -> impl Fn(Res<'_, AppTypeRegistry>) -> Result<Saved, LoadError> {
	let path = path.into();
	move |type_registry| {
		let input = std::fs::read(&path)?;
		let mut deserializer = ron::Deserializer::from_bytes(&input)?;
		let scene = {
			let type_registry = &type_registry.read();
			let scene_deserializer = SceneDeserializer { type_registry };
			scene_deserializer.deserialize(&mut deserializer)?
		};
		info!("loaded from file: {path:?}");
		Ok(Saved {
			scene,
			mapper: mapper.clone(),
		})
	}
}

pub fn read_file(
	In(path): In<PathBuf>,
	type_registry: Res<'_, AppTypeRegistry>,
) -> Result<Saved, LoadError> {
	let input = std::fs::read(&path)?;
	let mut deserializer = ron::Deserializer::from_bytes(&input)?;
	let scene = {
		let type_registry = &type_registry.read();
		let scene_deserializer = SceneDeserializer { type_registry };
		scene_deserializer.deserialize(&mut deserializer)?
	};
	info!("loaded from file: {path:?}");
	Ok(Saved {
		scene,
		mapper: SceneMapper::default(),
	})
}

pub fn read_stream<S: Read>(
	In(mut stream): In<S>,
	type_registry: Res<'_, AppTypeRegistry>,
) -> Result<Saved, LoadError> {
	let mut input = Vec::new();
	stream.read_to_end(&mut input)?;
	let mut deserializer = ron::Deserializer::from_bytes(&input)?;
	let scene = {
		let type_registry = &type_registry.read();
		let scene_deserializer = SceneDeserializer { type_registry };
		scene_deserializer.deserialize(&mut deserializer)?
	};
	info!("loaded from stream");
	Ok(Saved {
		scene,
		mapper: SceneMapper::default(),
	})
}

pub fn unload<Filter: QueryFilter>(
	In(result): In<Result<Saved, LoadError>>,
	world: &mut World,
) -> Result<Saved, LoadError> {
	let saved = result?;
	let entities = world
		.query_filtered::<Entity, Filter>()
		.iter(world)
		.collect::<Vec<_>>();
	for entity in entities {
		if let Ok(entity) = world.get_entity_mut(entity) {
			entity.despawn_recursive();
		}
	}

	Ok(saved)
}

pub fn write_to_world(
	In(result): In<Result<Saved, LoadError>>,
	world: &mut World,
) -> Result<Loaded, LoadError> {
	let Saved { scene, mut mapper } = result?;
	let mut entity_map = EntityHashMap::default();
	scene.write_to_world(world, &mut entity_map)?;
	if !mapper.is_empty() {
		for entity in entity_map.values().copied() {
			if let Ok(entity) = world.get_entity_mut(entity) {
				mapper.replace(entity);
			}
		}
	}

	Ok(Loaded { entity_map })
}

pub fn insert_into_loaded<B>(
	bundle: B,
) -> impl Fn(In<Result<Loaded, LoadError>>, &mut World) -> Result<Loaded, LoadError>
where
	B: Bundle + Clone,
{
	move |In(result), world| {
		if let Ok(loaded) = &result {
			for (saved_entity, entity) in &loaded.entity_map {
				if let Ok(mut entity) = world.get_entity_mut(*entity) {
					entity.insert(bundle.clone());
				} else {
					error!(
						"entity {saved_entity} is referenced in saved data but was never saved (raw bits = {})",
						saved_entity.to_bits()
					);
				}
			}
		}

		result
	}
}

pub fn insert_loaded(In(result): In<Result<Loaded, LoadError>>, world: &mut World) {
	match result {
		Ok(loaded) => world.insert_resource(loaded),
		Err(why) => error!("load failed: {why:?}"),
	}
}

#[must_use]
pub fn get_file_from_resource<R>(request: Res<'_, R>) -> PathBuf
where
	R: GetFilePath + Resource,
{
	request.path().to_owned()
}

pub fn get_file_from_event<E>(mut events: EventReader<'_, '_, E>) -> PathBuf
where
	E: Event + GetFilePath,
{
	let mut iter = events.read();
	let event = iter.next().unwrap();
	if iter.next().is_some() {
		warn!("multiple load request events received; only the first one is processed.");
	}

	event.path().to_owned()
}

pub fn get_stream_from_event<E>(mut events: EventReader<'_, '_, E>) -> E::Stream
where
	E: Event + GetStream,
{
	let mut iter = events.read();
	let event = iter.next().unwrap();
	if iter.next().is_some() {
		warn!("multiple load request events received; only the first one is processed.");
	}

	event.stream()
}

#[cfg(test)]
mod tests {
	use std::fs::*;

	use anyhow::Result;
	use bevy::prelude::*;

	use crate::{GetStaticStream, prelude::*, static_stream};

	pub const DATA: &str = "(
        resources: {},
        entities: {
            4294967296: (
                components: {
                    \"bevy_framework_save::load::tests::Dummy\": (),
                },
            ),
        },
    )";

	#[derive(Default, Component, Reflect)]
	#[reflect(Component)]
	struct Dummy;

	fn app() -> App {
		let mut app = App::new();
		app.add_plugins((MinimalPlugins, LoadPlugin))
			.register_type::<Dummy>();

		app
	}

	#[test]
	fn load_file() -> Result<()> {
		const PATH: &str = "test_load_file.ron";

		write(PATH, DATA)?;

		let mut app = app();
		app.add_systems(PreUpdate, load(static_file(PATH)));

		app.update();

		let world = app.world_mut();
		assert!(!world.contains_resource::<Loaded>());
		assert!(
			world
				.query_filtered::<(), With<Dummy>>()
				.get_single(world)
				.is_ok(),
		);

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn load_stream() -> Result<()> {
		const PATH: &str = "test_load_stream.ron";

		struct LoadStream;

		impl GetStaticStream for LoadStream {
			type Stream = File;

			fn stream() -> Self::Stream {
				File::open(PATH).unwrap()
			}
		}

		write(PATH, DATA)?;

		let mut app = app();
		app.add_systems(PreUpdate, load(static_stream(LoadStream)));

		app.update();

		let world = app.world_mut();
		assert!(!world.contains_resource::<Loaded>());
		assert!(
			world
				.query_filtered::<(), With<Dummy>>()
				.get_single(world)
				.is_ok()
		);

		remove_file(PATH)?;

		Ok(())
	}
}
