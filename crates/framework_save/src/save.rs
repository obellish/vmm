use std::{
	any::TypeId,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{self, Write},
	marker::PhantomData,
	path::PathBuf,
};

use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::{prelude::*, query::QueryFilter, schedule::SystemConfigs};
use bevy_framework_utils::prelude::*;
use bevy_scene::{DynamicScene, DynamicSceneBuilder, SceneFilter};
use bevy_utils::{
	HashSet,
	tracing::{error, info, warn},
};

use super::{
	FileFromEvent, FileFromResource, GetFilePath, GetStaticStream, GetStream, MapComponent,
	Pipeline, SceneMapper, StaticFile, StaticStream, StreamFromEvent, StreamFromResource,
};

pub struct SavePlugin;

impl Plugin for SavePlugin {
	fn build(&self, app: &mut App) {
		app.configure_sets(
			PreUpdate,
			(
				SaveSet::Save,
				SaveSet::PostSave.run_if(has_resource::<Saved>),
			)
				.chain(),
		)
		.add_systems(
			PreUpdate,
			remove_resource::<Saved>.in_set(SaveSet::PostSave),
		);
	}
}

#[derive(Default, Clone, Component)]
pub struct Save;

#[derive(Resource)]
pub struct Saved {
	pub scene: DynamicScene,
	pub mapper: SceneMapper,
}

#[derive(Clone)]
pub struct SaveInput {
	pub entities: EntityFilter,
	pub resources: SceneFilter,
	pub components: SceneFilter,
	pub mapper: SceneMapper,
}

impl Default for SaveInput {
	fn default() -> Self {
		Self {
			entities: EntityFilter::any(),
			components: SceneFilter::allow_all(),
			resources: SceneFilter::deny_all(),
			mapper: SceneMapper::default(),
		}
	}
}

#[repr(transparent)]
pub struct SavePipelineBuilder<F: QueryFilter> {
	query: PhantomData<F>,
	input: SaveInput,
}

impl<F> SavePipelineBuilder<F>
where
	F: QueryFilter + 'static,
{
	#[must_use]
	pub fn include_resource<R: Resource>(mut self) -> Self {
		self.input.resources = self.input.resources.allow::<R>();
		self
	}

	#[must_use]
	pub fn include_resource_by_id(mut self, type_id: TypeId) -> Self {
		self.input.resources = self.input.resources.allow_by_id(type_id);
		self
	}

	#[must_use]
	pub fn exclude_component<T: Component>(mut self) -> Self {
		self.input.components = self.input.components.deny::<T>();
		self
	}

	#[must_use]
	pub fn exclude_component_by_id(mut self, type_id: TypeId) -> Self {
		self.input.components = self.input.components.deny_by_id(type_id);
		self
	}

	#[must_use]
	pub fn map_component<T: Component>(mut self, m: impl MapComponent<T>) -> Self {
		self.input.mapper = self.input.mapper.map(m);
		self
	}

	pub fn into(self, p: impl SavePipeline) -> SystemConfigs {
		let Self { input, .. } = self;
		let system = (move || input.clone())
			.pipe(filter_entities::<F>)
			.pipe(map_scene)
			.pipe(save_scene);
		let system = p
			.save(IntoSystem::into_system(system))
			.pipe(unmap_scene)
			.pipe(insert_saved);

		p.finish(IntoSystem::into_system(system))
			.in_set(SaveSet::Save)
	}
}

#[repr(transparent)]
pub struct DynamicSavePipelineBuilder<F: QueryFilter, S>
where
	S: System<In = (), Out = SaveInput>,
{
	query: PhantomData<F>,
	input_source: S,
}

impl<F, S> DynamicSavePipelineBuilder<F, S>
where
	F: QueryFilter + 'static,
	S: System<In = (), Out = SaveInput>,
{
	pub fn into(self, p: impl SavePipeline) -> SystemConfigs {
		let Self { input_source, .. } = self;
		let system = input_source
			.pipe(filter_entities::<F>)
			.pipe(map_scene)
			.pipe(save_scene);
		let system = p
			.save(IntoSystem::into_system(system))
			.pipe(unmap_scene)
			.pipe(insert_saved);

		p.finish(IntoSystem::into_system(system))
			.in_set(SaveSet::Save)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum SaveSet {
	Save,
	PostSave,
}

#[derive(Debug)]
pub enum SaveError {
	Ron(ron::Error),
	Io(io::Error),
}

impl Display for SaveError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Ron(r) => Display::fmt(&r, f),
			Self::Io(i) => Display::fmt(&i, f),
		}
	}
}

impl StdError for SaveError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Ron(r) => Some(r),
			Self::Io(i) => Some(i),
		}
	}
}

impl From<io::Error> for SaveError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<ron::Error> for SaveError {
	fn from(value: ron::Error) -> Self {
		Self::Ron(value)
	}
}

#[derive(Default, Clone)]
pub enum EntityFilter {
	#[default]
	Any,
	Allow(HashSet<Entity>),
	Block(HashSet<Entity>),
}

impl EntityFilter {
	#[must_use]
	pub const fn any() -> Self {
		Self::Any
	}

	pub fn allow(entities: impl IntoIterator<Item = Entity>) -> Self {
		Self::Allow(entities.into_iter().collect())
	}

	pub fn block(entities: impl IntoIterator<Item = Entity>) -> Self {
		Self::Block(entities.into_iter().collect())
	}
}

pub trait SavePipeline: Pipeline {
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>>;
}

impl<E> SavePipeline for FileFromEvent<E>
where
	E: Event + GetFilePath,
{
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>> {
		IntoSystem::into_system(system.pipe(get_file_from_event::<E>).pipe(write_file))
	}
}

impl<R> SavePipeline for FileFromResource<R>
where
	R: GetFilePath + Resource,
{
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>> {
		IntoSystem::into_system(system.pipe(get_file_from_resource::<R>).pipe(write_file))
	}
}

impl SavePipeline for StaticFile {
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>> {
		IntoSystem::into_system(system.pipe(write_static_file(self.0.clone())))
	}
}

impl<S: GetStaticStream> SavePipeline for StaticStream<S>
where
	S::Stream: Write,
{
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>> {
		IntoSystem::into_system(
			system
				.pipe(move |In(saved): In<Saved>| (S::stream(), saved))
				.pipe(write_stream),
		)
	}
}

impl<E> SavePipeline for StreamFromEvent<E>
where
	E: Event + GetStream,
	E::Stream: Write,
{
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>> {
		IntoSystem::into_system(system.pipe(get_stream_from_event::<E>).pipe(write_stream))
	}
}

impl<R> SavePipeline for StreamFromResource<R>
where
	R: GetStream + Resource,
	R::Stream: Write,
{
	fn save(
		&self,
		system: impl System<In = (), Out = Saved>,
	) -> impl System<In = (), Out = Result<Saved, SaveError>> {
		IntoSystem::into_system(
			system
				.pipe(move |In(saved): In<Saved>, resource: Res<'_, R>| (resource.stream(), saved))
				.pipe(write_stream),
		)
	}
}

#[must_use]
pub fn filter<F: QueryFilter>(entities: Query<'_, '_, Entity, F>) -> SaveInput {
	SaveInput {
		entities: EntityFilter::allow(&entities),
		resources: SceneFilter::deny_all(),
		..SaveInput::default()
	}
}

#[must_use]
pub fn filter_entities<F>(
	In(mut input): In<SaveInput>,
	entities: Query<'_, '_, Entity, F>,
) -> SaveInput
where
	F: QueryFilter + 'static,
{
	input.entities = EntityFilter::allow(&entities);
	input
}

pub fn map_scene(In(mut input): In<SaveInput>, world: &mut World) -> SaveInput {
	if !input.mapper.is_empty() {
		match &input.entities {
			EntityFilter::Any => {
				let entities = world
					.iter_entities()
					.map(|entity| entity.id())
					.collect::<Vec<_>>();
				for entity in entities {
					input.mapper.apply(world.entity_mut(entity));
				}
			}
			EntityFilter::Allow(entities) => {
				for entity in entities {
					input.mapper.apply(world.entity_mut(*entity));
				}
			}
			EntityFilter::Block(blocked) => {
				let entities = world
					.iter_entities()
					.filter_map(|entity| (!blocked.contains(&entity.id())).then_some(entity.id()))
					.collect::<Vec<_>>();

				for entity in entities {
					input.mapper.apply(world.entity_mut(entity));
				}
			}
		}
	}

	input
}

pub fn save_scene(In(input): In<SaveInput>, world: &World) -> Saved {
	let mut builder = DynamicSceneBuilder::from_world(world)
		.with_component_filter(input.components)
		.with_resource_filter(input.resources)
		.extract_resources();

	match input.entities {
		EntityFilter::Any => {}
		EntityFilter::Allow(entities) => {
			builder = builder.extract_entities(entities.into_iter());
		}
		EntityFilter::Block(entities) => {
			builder =
				builder.extract_entities(world.iter_entities().filter_map(|entity| {
					(!entities.contains(&entity.id())).then_some(entity.id())
				}));
		}
	}

	Saved {
		scene: builder.build(),
		mapper: input.mapper,
	}
}

pub fn write_static_file(
	path: PathBuf,
) -> impl Fn(In<Saved>, Res<'_, AppTypeRegistry>) -> Result<Saved, SaveError> {
	move |In(saved), type_registry| {
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		let data = saved.scene.serialize(&type_registry.read())?;
		std::fs::write(&path, data.as_bytes())?;
		info!("saved intop file: {path:?}");
		Ok(saved)
	}
}

pub fn write_file(
	In((path, saved)): In<(PathBuf, Saved)>,
	type_registry: Res<'_, AppTypeRegistry>,
) -> Result<Saved, SaveError> {
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	let data = saved.scene.serialize(&type_registry.read())?;
	std::fs::write(&path, data.as_bytes())?;
	info!("saved into file: {path:?}");
	Ok(saved)
}

pub fn write_stream<S: Write>(
	In((mut stream, saved)): In<(S, Saved)>,
	type_registry: Res<'_, AppTypeRegistry>,
) -> Result<Saved, SaveError> {
	let data = saved.scene.serialize(&type_registry.read())?;
	stream.write_all(data.as_bytes())?;
	info!("saved into stream");
	Ok(saved)
}

pub fn unmap_scene(
	In(mut result): In<Result<Saved, SaveError>>,
	world: &mut World,
) -> Result<Saved, SaveError> {
	if let Ok(saved) = &mut result {
		if !saved.mapper.is_empty() {
			for entity in saved.scene.entities.iter().map(|e| e.entity) {
				saved.mapper.undo(world.entity_mut(entity));
			}
		}
	}

	result
}

pub fn insert_saved(In(result): In<Result<Saved, SaveError>>, world: &mut World) {
	match result {
		Ok(saved) => world.insert_resource(saved),
		Err(why) => error!("save failed: {why:?}"),
	}
}

#[must_use]
pub fn get_file_from_resource<R>(In(saved): In<Saved>, request: Res<'_, R>) -> (PathBuf, Saved)
where
	R: GetFilePath + Resource,
{
	let path = request.path().to_owned();
	(path, saved)
}

pub fn get_file_from_event<E>(
	In(saved): In<Saved>,
	mut events: EventReader<'_, '_, E>,
) -> (PathBuf, Saved)
where
	E: Event + GetFilePath,
{
	let mut iter = events.read();
	let event = iter.next().unwrap();
	if iter.next().is_some() {
		warn!("multiple save request events received; only the first one is processed.");
	}
	let path = event.path().to_owned();
	(path, saved)
}

pub fn get_stream_from_event<E>(
	In(saved): In<Saved>,
	mut events: EventReader<'_, '_, E>,
) -> (<E as GetStream>::Stream, Saved)
where
	E: Event + GetStream,
{
	let mut iter = events.read();
	let event = iter.next().unwrap();
	if iter.next().is_some() {
		warn!("multiple save request events received; only the first one is processed.");
	}
	(event.stream(), saved)
}

#[must_use]
pub fn save<F: QueryFilter>() -> SavePipelineBuilder<F> {
	SavePipelineBuilder {
		query: PhantomData,
		input: SaveInput::default(),
	}
}

#[must_use]
pub fn save_default() -> SavePipelineBuilder<With<Save>> {
	save()
}

#[must_use]
pub fn save_all() -> SavePipelineBuilder<()> {
	save()
}

pub fn save_with<F: QueryFilter, S, M>(input_source: S) -> DynamicSavePipelineBuilder<F, S::System>
where
	S: IntoSystem<(), SaveInput, M>,
{
	DynamicSavePipelineBuilder {
		query: PhantomData,
		input_source: IntoSystem::into_system(input_source),
	}
}

pub fn save_default_with<S, M>(s: S) -> DynamicSavePipelineBuilder<With<Save>, S::System>
where
	S: IntoSystem<(), SaveInput, M>,
{
	save_with(s)
}

pub fn save_all_with<S, M>(s: S) -> DynamicSavePipelineBuilder<(), S::System>
where
	S: IntoSystem<(), SaveInput, M>,
{
	save_with(s)
}

#[cfg(test)]
mod tests {
	use std::{fs::*, path::Path};

	use anyhow::Result;
	use bevy::prelude::*;

	use crate::{
		GetStaticStream, GetStream, prelude::*, static_stream, stream_from_event,
		stream_from_resource,
	};

	#[derive(Default, Component, Reflect)]
	#[reflect(Component)]
	struct Dummy;

	#[derive(Default, Component, Reflect)]
	#[reflect(Component)]
	struct Foo;

	fn app() -> App {
		let mut app = App::new();
		app.add_plugins((MinimalPlugins, SavePlugin))
			.register_type::<Dummy>();

		app
	}

	#[test]
	fn save_into_file() -> Result<()> {
		const PATH: &str = "test_save_into_file.ron";
		let mut app = app();
		app.add_systems(PreUpdate, save_default().into(static_file(PATH)));

		app.world_mut().spawn((Dummy, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));
		assert!(!app.world().contains_resource::<Saved>());

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_into_stream() -> Result<()> {
		const PATH: &str = "test_save_into_stream.ron";

		struct SaveStream;

		impl GetStaticStream for SaveStream {
			type Stream = File;

			fn stream() -> Self::Stream {
				File::create(PATH).unwrap()
			}
		}

		let mut app = app();
		app.add_systems(PreUpdate, save_default().into(static_stream(SaveStream)));

		app.world_mut().spawn((Dummy, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));
		assert!(!app.world().contains_resource::<Saved>());

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_into_file_from_resource() -> Result<()> {
		const PATH: &str = "test_save_into_file_from_resource.ron";

		#[derive(Resource)]
		struct SaveRequest;

		impl GetFilePath for SaveRequest {
			fn path(&self) -> &Path {
				PATH.as_ref()
			}
		}

		let mut app = app();
		app.add_systems(
			PreUpdate,
			save_default().into(file_from_resource::<SaveRequest>()),
		);

		app.world_mut().insert_resource(SaveRequest);
		app.world_mut().spawn((Dummy, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));
		assert!(!app.world().contains_resource::<SaveRequest>());

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_into_stream_from_resource() -> Result<()> {
		const PATH: &str = "test_save_into_stream_from_resource.ron";

		#[derive(Resource)]
		struct SaveRequest(&'static str);

		impl GetStream for SaveRequest {
			type Stream = File;

			fn stream(&self) -> Self::Stream {
				File::create(self.0).unwrap()
			}
		}

		let mut app = app();
		app.add_systems(
			PreUpdate,
			save_default().into(stream_from_resource::<SaveRequest>()),
		);

		app.world_mut().insert_resource(SaveRequest(PATH));
		app.world_mut().spawn((Dummy, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));
		assert!(!app.world().contains_resource::<Saved>());
		assert!(!app.world().contains_resource::<SaveRequest>());

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_into_file_from_event() -> Result<()> {
		const PATH: &str = "test_save_into_file_from_event.ron";

		#[derive(Event)]
		struct SaveRequest;

		impl GetFilePath for SaveRequest {
			fn path(&self) -> &Path {
				PATH.as_ref()
			}
		}

		let mut app = app();
		app.add_event::<SaveRequest>().add_systems(
			PreUpdate,
			save_default().into(file_from_event::<SaveRequest>()),
		);

		app.world_mut().send_event(SaveRequest);
		app.world_mut().spawn((Dummy, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_into_stream_from_event() -> Result<()> {
		const PATH: &str = "test_save_into_stream_from_event.ron";

		#[derive(Event)]
		struct SaveRequest(&'static str);

		impl GetStream for SaveRequest {
			type Stream = File;

			fn stream(&self) -> Self::Stream {
				File::create(self.0).unwrap()
			}
		}

		let mut app = app();

		app.add_event::<SaveRequest>().add_systems(
			PreUpdate,
			save_default().into(stream_from_event::<SaveRequest>()),
		);

		app.world_mut().send_event(SaveRequest(PATH));
		app.world_mut().spawn((Dummy, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_resource() -> Result<()> {
		const PATH: &str = "test_save_resource.ron";

		#[derive(Default, Reflect, Resource)]
		#[reflect(Resource)]
		struct Dummy;

		let mut app = app();
		app.register_type::<Dummy>()
			.insert_resource(Dummy)
			.add_systems(
				Update,
				save_default()
					.include_resource::<Dummy>()
					.into(static_file(PATH)),
			);

		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_without_component() -> Result<()> {
		const PATH: &str = "test_save_without_component.ron";

		let mut app = app();
		app.add_systems(
			PreUpdate,
			save_default()
				.exclude_component::<Foo>()
				.into(static_file(PATH)),
		);

		app.world_mut().spawn((Dummy, Foo, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));
		assert!(!data.contains("Foo"));

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_without_component_dynamically() -> Result<()> {
		const PATH: &str = "test_save_without_component_dynamically.ron";

		fn deny_foo() -> SaveInput {
			SaveInput {
				components: SceneFilter::default().deny::<Foo>(),
				..default()
			}
		}

		let mut app = app();
		app.add_systems(
			PreUpdate,
			save_default_with(deny_foo).into(static_file(PATH)),
		);

		app.world_mut().spawn((Dummy, Foo, Save));
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Dummy"));
		assert!(!data.contains("Foo"));

		remove_file(PATH)?;

		Ok(())
	}

	#[test]
	fn save_map_component() -> Result<()> {
		const PATH: &str = "test_save_map_component.ron";

		#[derive(Default, Component)]
		struct Foo(#[allow(dead_code)] u32);

		#[derive(Default, Component, Reflect)]
		#[reflect(Component)]
		struct Bar(u32);

		let mut app = app();
		app.register_type::<Bar>().add_systems(
			PreUpdate,
			save_default()
				.map_component::<Foo>(|Foo(i): &Foo| Bar(*i))
				.into(static_file(PATH)),
		);

		let entity = app.world_mut().spawn((Foo(12), Save)).id();
		app.update();

		let data = read_to_string(PATH)?;
		assert!(data.contains("Bar"));
		assert!(data.contains("(12)"));
		assert!(!data.contains("Foo"));
		assert!(app.world().entity(entity).contains::<Foo>());
		assert!(!app.world().entity(entity).contains::<Bar>());

		remove_file(PATH)?;

		Ok(())
	}
}
