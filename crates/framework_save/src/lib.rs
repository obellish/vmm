#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod load;
pub mod prelude;
pub mod save;

use std::{
	marker::PhantomData,
	path::{Path, PathBuf},
};

use bevy_ecs::{prelude::*, schedule::SystemConfigs};
use bevy_framework_utils::prelude::*;

#[repr(transparent)]
pub struct FileFromEvent<E>(PhantomData<E>);

impl<E: Event> Pipeline for FileFromEvent<E> {}

#[repr(transparent)]
pub struct FileFromResource<R>(PhantomData<R>);

impl<R: Resource> Pipeline for FileFromResource<R> {
	fn finish(&self, pipeline: impl System<In = (), Out = ()>) -> SystemConfigs {
		pipeline
			.pipe(remove_resource::<R>)
			.run_if(has_resource::<R>)
	}
}

#[repr(transparent)]
pub struct StaticFile(PathBuf);

impl Pipeline for StaticFile {}

#[derive(Clone)]
#[repr(transparent)]
pub struct StaticStream<S>(S);

impl<S> Pipeline for StaticStream<S> where S: Send + Sync + 'static {}

#[repr(transparent)]
pub struct StreamFromEvent<E>(PhantomData<E>);

impl<E: Event> Pipeline for StreamFromEvent<E> {}

#[repr(transparent)]
pub struct StreamFromResource<R>(PhantomData<R>);

impl<R: Resource> Pipeline for StreamFromResource<R> {
	fn finish(&self, pipeline: impl System<In = (), Out = ()>) -> SystemConfigs {
		pipeline
			.pipe(remove_resource::<R>)
			.run_if(has_resource::<R>)
	}
}

#[derive(Default)]
#[repr(transparent)]
pub struct SceneMapper(Vec<ComponentMapperDyn>);

impl SceneMapper {
	#[must_use]
	pub fn map<T: Component>(mut self, m: impl MapComponent<T>) -> Self {
		self.0.push(Box::new(ComponentMapperImpl::new(m)));
		self
	}

	pub(crate) fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub(crate) fn apply(&mut self, mut entity: EntityWorldMut<'_>) {
		for mapper in &mut self.0 {
			mapper.apply(&mut entity);
		}
	}

	pub(crate) fn replace(&mut self, mut entity: EntityWorldMut<'_>) {
		for mapper in &mut self.0 {
			mapper.replace(&mut entity);
		}
	}

	pub(crate) fn undo(&mut self, mut entity: EntityWorldMut<'_>) {
		for mapper in &mut self.0 {
			mapper.undo(&mut entity);
		}
	}
}

impl Clone for SceneMapper {
	fn clone(&self) -> Self {
		Self(self.0.iter().map(|mapper| mapper.clone_dyn()).collect())
	}
}

#[repr(transparent)]
struct ComponentMapperImpl<T: Component, M>(M, PhantomData<T>)
where
	M: MapComponent<T>;

impl<T: Component, M> ComponentMapperImpl<T, M>
where
	M: MapComponent<T>,
{
	const fn new(m: M) -> Self {
		Self(m, PhantomData)
	}
}

impl<T: Component, M> ComponentMapper for ComponentMapperImpl<T, M>
where
	M: MapComponent<T>,
{
	fn apply(&mut self, entity: &mut EntityWorldMut<'_>) {
		if let Some(component) = entity.get::<T>() {
			entity.insert(self.0.map_component(component));
		}
	}

	fn replace(&mut self, entity: &mut EntityWorldMut<'_>) {
		if let Some(component) = entity.take::<T>() {
			entity.insert(self.0.map_component(&component));
		}
	}

	fn undo(&mut self, entity: &mut EntityWorldMut<'_>) {
		entity.remove::<M::Output>();
	}

	fn clone_dyn(&self) -> ComponentMapperDyn {
		Box::new(Self::new(self.0.clone()))
	}
}

pub trait GetFilePath {
	fn path(&self) -> &Path;
}

pub trait GetStaticStream: Send + Sync + 'static {
	type Stream: Send + Sync + 'static;

	fn stream() -> Self::Stream;
}

pub trait GetStream: Send + Sync + 'static {
	type Stream: Send + Sync + 'static;

	fn stream(&self) -> Self::Stream;
}

pub trait MapComponent<T: Component>: Clone + Send + Sync + 'static {
	type Output: Component;

	fn map_component(&self, component: &T) -> Self::Output;
}

impl<F, T: Component, U: Component> MapComponent<T> for F
where
	F: Fn(&T) -> U,
	F: Clone + Send + Sync + 'static,
{
	type Output = U;

	fn map_component(&self, component: &T) -> Self::Output {
		self(component)
	}
}

pub trait Pipeline: Send + Sync + 'static {
	fn finish(&self, pipeline: impl System<In = (), Out = ()>) -> SystemConfigs {
		pipeline.into_configs()
	}
}

trait ComponentMapper: Send + Sync + 'static {
	fn apply(&mut self, entity: &mut EntityWorldMut<'_>);

	fn replace(&mut self, entity: &mut EntityWorldMut<'_>);

	fn undo(&mut self, entity: &mut EntityWorldMut<'_>);

	fn clone_dyn(&self) -> ComponentMapperDyn;
}

type ComponentMapperDyn = Box<dyn ComponentMapper>;

#[must_use]
pub const fn file_from_event<E: Event>() -> FileFromEvent<E> {
	FileFromEvent(PhantomData)
}

#[must_use]
pub const fn file_from_resource<R: Resource>() -> FileFromResource<R> {
	FileFromResource(PhantomData)
}

pub fn static_file(path: impl Into<PathBuf>) -> StaticFile {
	StaticFile(path.into())
}

pub const fn static_stream<S>(stream: S) -> StaticStream<S> {
	StaticStream(stream)
}

#[must_use]
pub const fn stream_from_event<E: Event>() -> StreamFromEvent<E> {
	StreamFromEvent(PhantomData)
}

#[must_use]
pub const fn stream_from_resource<R: Resource>() -> StreamFromResource<R> {
	StreamFromResource(PhantomData)
}
