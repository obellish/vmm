use std::{
	cmp::Ordering,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hash::{Hash, Hasher},
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use bevy_ecs::{
	archetype::Archetype,
	component::{ComponentId, Components, Tick},
	entity::{EntityMapper, MapEntities},
	prelude::*,
	query::{FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery},
	storage::{Table, TableRow},
	system::EntityCommands,
	world::unsafe_world_cell::UnsafeWorldCell,
};
use bevy_reflect::Reflect;

use super::{Any, CastInto, Kind};

#[derive(Reflect)]
#[repr(transparent)]
pub struct Instance<T: Kind>(Entity, #[reflect(ignore)] PhantomData<T>);

impl<T: Kind> Instance<T> {
	pub const PLACEHOLDER: Self = Self(Entity::PLACEHOLDER, PhantomData);

	#[must_use]
	pub const unsafe fn from_entity_unchecked(entity: Entity) -> Self {
		Self(entity, PhantomData)
	}

	#[must_use]
	pub const fn entity(self) -> Entity {
		self.0
	}

	#[must_use]
	pub fn cast_into<U: Kind>(self) -> Instance<U>
	where
		T: CastInto<U>,
	{
		T::cast_into(self)
	}

	#[must_use]
	pub const fn cast_into_any(self) -> Instance<Any> {
		unsafe { self.cast_into_unchecked() }
	}

	#[must_use]
	pub const unsafe fn cast_into_unchecked<U: Kind>(self) -> Instance<U> {
		unsafe { Instance::from_entity_unchecked(self.entity()) }
	}
}

impl<T: Component> Instance<T> {
	#[must_use]
	pub fn from_entity(entity: EntityRef<'_>) -> Option<Self> {
		if entity.contains::<T>() {
			Some(unsafe { Self::from_entity_unchecked(entity.id()) })
		} else {
			None
		}
	}
}

impl<T: Kind> Clone for Instance<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Kind> Copy for Instance<T> {}

impl<T: Kind> Debug for Instance<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&T::debug_name())?;
		f.write_char('(')?;
		Display::fmt(&self.0.index(), f)?;
		f.write_char('v')?;
		Display::fmt(&self.0.generation(), f)?;
		f.write_char(')')
	}
}

impl<T: Kind> Deref for Instance<T> {
	type Target = Entity;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Kind> Eq for Instance<T> {}

impl<T: Component> From<InstanceRef<'_, T>> for Instance<T> {
	fn from(value: InstanceRef<'_, T>) -> Self {
		value.instance()
	}
}

impl<T: Component> From<&InstanceRef<'_, T>> for Instance<T> {
	fn from(value: &InstanceRef<'_, T>) -> Self {
		value.instance()
	}
}

impl<T: Component> From<InstanceMutItem<'_, T>> for Instance<T> {
	fn from(value: InstanceMutItem<'_, T>) -> Self {
		value.instance()
	}
}

impl<T: Component> From<&InstanceMutItem<'_, T>> for Instance<T> {
	fn from(value: &InstanceMutItem<'_, T>) -> Self {
		value.instance()
	}
}

impl<T: Component> From<InstanceMutReadOnlyItem<'_, T>> for Instance<T> {
	fn from(value: InstanceMutReadOnlyItem<'_, T>) -> Self {
		value.instance()
	}
}

impl<T: Component> From<&InstanceMutReadOnlyItem<'_, T>> for Instance<T> {
	fn from(value: &InstanceMutReadOnlyItem<'_, T>) -> Self {
		value.instance()
	}
}

impl<'a, T: Kind> From<InstanceCommands<'a, T>> for Instance<T> {
	fn from(value: InstanceCommands<'a, T>) -> Self {
		value.instance()
	}
}

impl<'a, T: Kind> From<&InstanceCommands<'a, T>> for Instance<T> {
	fn from(value: &InstanceCommands<'a, T>) -> Self {
		value.instance()
	}
}

impl From<Entity> for Instance<Any> {
	fn from(value: Entity) -> Self {
		Self(value, PhantomData)
	}
}

impl<T: Kind> From<Instance<T>> for Entity {
	fn from(value: Instance<T>) -> Self {
		value.entity()
	}
}

impl<T: Kind> Hash for Instance<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<T: Kind> MapEntities for Instance<T> {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.0 = entity_mapper.map_entity(self.0);
	}
}

impl<T: Kind> Ord for Instance<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.cmp(&other.0)
	}
}

impl<T: Kind> PartialEq for Instance<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<T: Kind> PartialEq<Entity> for Instance<T> {
	fn eq(&self, other: &Entity) -> bool {
		self.0 == *other
	}
}

impl<T: Kind> PartialOrd for Instance<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

unsafe impl<T: Kind> QueryData for Instance<T> {
	type ReadOnly = Self;
}

unsafe impl<T: Kind> ReadOnlyQueryData for Instance<T> {}

unsafe impl<T: Kind> WorldQuery for Instance<T> {
	type Fetch<'a> = <T::Filter as WorldQuery>::Fetch<'a>;
	type Item<'a> = Self;
	type State = <T::Filter as WorldQuery>::State;

	const IS_DENSE: bool = <T::Filter as WorldQuery>::IS_DENSE;

	fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
		item
	}

	fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
		<T::Filter as WorldQuery>::shrink_fetch(fetch)
	}

	unsafe fn init_fetch<'w>(
		world: UnsafeWorldCell<'w>,
		state: &Self::State,
		last_run: Tick,
		this_run: Tick,
	) -> Self::Fetch<'w> {
		unsafe { <T::Filter as WorldQuery>::init_fetch(world, state, last_run, this_run) }
	}

	unsafe fn set_archetype<'w>(
		_: &mut Self::Fetch<'w>,
		_: &Self::State,
		_: &'w Archetype,
		_: &'w Table,
	) {
	}

	unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
		unsafe { <T::Filter as WorldQuery>::set_table(fetch, state, table) };
	}

	unsafe fn fetch<'w>(_: &mut Self::Fetch<'w>, entity: Entity, _: TableRow) -> Self::Item<'w> {
		unsafe { Self::from_entity_unchecked(entity) }
	}

	fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
		<T::Filter as WorldQuery>::update_component_access(state, access);
	}

	fn get_state(components: &Components) -> Option<Self::State> {
		<T::Filter as WorldQuery>::get_state(components)
	}

	fn init_state(world: &mut World) -> Self::State {
		<T::Filter as WorldQuery>::init_state(world)
	}

	fn matches_component_set(
		state: &Self::State,
		set_contains_id: &impl Fn(ComponentId) -> bool,
	) -> bool {
		<T::Filter as WorldQuery>::matches_component_set(state, set_contains_id)
	}
}

pub struct InstanceRef<'a, T: Component> {
	instance: Instance<T>,
	data: &'a T,
}

impl<'a, T: Component> InstanceRef<'a, T> {
	#[must_use]
	pub fn from_entity(entity: EntityRef<'a>) -> Option<Self> {
		Some(Self {
			data: entity.get()?,
			instance: unsafe { Instance::from_entity_unchecked(entity.id()) },
		})
	}

	#[must_use]
	pub const fn entity(self) -> Entity {
		self.instance().entity()
	}

	#[must_use]
	pub const fn instance(self) -> Instance<T> {
		self.instance
	}
}

impl<T: Component> AsRef<T> for InstanceRef<'_, T> {
	fn as_ref(&self) -> &T {
		self.data
	}
}

impl<T: Component> AsRef<Instance<T>> for InstanceRef<'_, T> {
	fn as_ref(&self) -> &Instance<T> {
		&self.instance
	}
}

impl<T: Component> Clone for InstanceRef<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Component> Copy for InstanceRef<'_, T> {}

impl<T: Component> Debug for InstanceRef<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.instance(), f)
	}
}

impl<T: Component> Deref for InstanceRef<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.data
	}
}

impl<T: Component> Eq for InstanceRef<'_, T> {}

impl<T: Component> PartialEq for InstanceRef<'_, T> {
	fn eq(&self, other: &Self) -> bool {
		self.instance == other.instance
	}
}

unsafe impl<T: Component> QueryData for InstanceRef<'_, T> {
	type ReadOnly = Self;
}

unsafe impl<T: Component> ReadOnlyQueryData for InstanceRef<'_, T> {}

unsafe impl<T: Component> WorldQuery for InstanceRef<'_, T> {
	type Fetch<'a> = <(Instance<T>, &'static T) as WorldQuery>::Fetch<'a>;
	type Item<'a> = InstanceRef<'a, T>;
	type State = <(Instance<T>, &'static T) as WorldQuery>::State;

	const IS_DENSE: bool = <(Instance<T>, &T) as WorldQuery>::IS_DENSE;

	fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
		item
	}

	fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
		<(Instance<T>, &T) as WorldQuery>::shrink_fetch(fetch)
	}

	unsafe fn init_fetch<'w>(
		world: UnsafeWorldCell<'w>,
		state: &Self::State,
		last_run: Tick,
		this_run: Tick,
	) -> Self::Fetch<'w> {
		unsafe { <(Instance<T>, &T) as WorldQuery>::init_fetch(world, state, last_run, this_run) }
	}

	unsafe fn set_archetype<'w>(
		fetch: &mut Self::Fetch<'w>,
		state: &Self::State,
		archetype: &'w Archetype,
		table: &'w Table,
	) {
		unsafe { <(Instance<T>, &T) as WorldQuery>::set_archetype(fetch, state, archetype, table) };
	}

	unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
		unsafe { <(Instance<T>, &T) as WorldQuery>::set_table(fetch, state, table) };
	}

	unsafe fn fetch<'w>(
		fetch: &mut Self::Fetch<'w>,
		entity: Entity,
		table_row: TableRow,
	) -> Self::Item<'w> {
		let (instance, data) =
			unsafe { <(Instance<T>, &T) as WorldQuery>::fetch(fetch, entity, table_row) };
		Self::Item { instance, data }
	}

	fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
		<(Instance<T>, &T) as WorldQuery>::update_component_access(state, access);
	}

	fn init_state(world: &mut World) -> Self::State {
		<(Instance<T>, &T) as WorldQuery>::init_state(world)
	}

	fn get_state(components: &Components) -> Option<Self::State> {
		<(Instance<T>, &T) as WorldQuery>::get_state(components)
	}

	fn matches_component_set(
		state: &Self::State,
		set_contains_id: &impl Fn(ComponentId) -> bool,
	) -> bool {
		<(Instance<T>, &T) as WorldQuery>::matches_component_set(state, set_contains_id)
	}
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct InstanceMut<T: Component> {
	instance: Instance<T>,
	data: &'static mut T,
}

impl<'a, T: Component> InstanceMutItem<'a, T> {
	pub fn from_entity(world: &'a mut World, entity: Entity) -> Option<Self> {
		world.get_mut(entity).map(|data| Self {
			data,
			instance: unsafe { Instance::from_entity_unchecked(entity) },
		})
	}

	#[must_use]
	pub const fn entity(&self) -> Entity {
		self.instance().entity()
	}

	#[must_use]
	pub const fn instance(&self) -> Instance<T> {
		self.instance
	}
}

impl<T: Component> AsMut<T> for InstanceMutItem<'_, T> {
	fn as_mut(&mut self) -> &mut T {
		self.data.as_mut()
	}
}

impl<T: Component> AsRef<T> for InstanceMutItem<'_, T> {
	fn as_ref(&self) -> &T {
		self.data.as_ref()
	}
}

impl<T: Component> AsRef<Instance<T>> for InstanceMutItem<'_, T> {
	fn as_ref(&self) -> &Instance<T> {
		&self.instance
	}
}

impl<T: Component> Debug for InstanceMutItem<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.instance(), f)
	}
}

impl<T: Component> Deref for InstanceMutItem<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.data.as_ref()
	}
}

impl<T: Component> DerefMut for InstanceMutItem<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.data.as_mut()
	}
}

impl<T: Component> Eq for InstanceMutItem<'_, T> {}

impl<T: Component> PartialEq for InstanceMutItem<'_, T> {
	fn eq(&self, other: &Self) -> bool {
		self.instance == other.instance
	}
}

impl<T: Component> InstanceMutReadOnlyItem<'_, T> {
	#[must_use]
	pub const fn entity(&self) -> Entity {
		self.instance().entity()
	}

	#[must_use]
	pub const fn instance(&self) -> Instance<T> {
		self.instance
	}
}

impl<T: Component> Debug for InstanceMutReadOnlyItem<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.instance(), f)
	}
}

impl<T: Component> Deref for InstanceMutReadOnlyItem<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.data
	}
}

impl<T: Component> Eq for InstanceMutReadOnlyItem<'_, T> {}

impl<T: Component> PartialEq for InstanceMutReadOnlyItem<'_, T> {
	fn eq(&self, other: &Self) -> bool {
		self.instance == other.instance
	}
}

#[repr(transparent)]
pub struct InstanceCommands<'a, T: Kind>(EntityCommands<'a>, PhantomData<T>);

impl<'a, T: Kind> InstanceCommands<'a, T> {
	#[must_use]
	pub const unsafe fn from_entity_unchecked(entity: EntityCommands<'a>) -> Self {
		Self(entity, PhantomData)
	}

	#[must_use]
	pub fn instance(&self) -> Instance<T> {
		unsafe { Instance::from_entity_unchecked(self.entity()) }
	}

	#[must_use]
	pub fn entity(&self) -> Entity {
		self.0.id()
	}

	pub const fn as_entity(&mut self) -> &mut EntityCommands<'a> {
		&mut self.0
	}

	pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
		self.0.insert(bundle);
		self
	}

	pub fn remove<U: Component>(&mut self) -> &mut Self {
		self.0.remove::<U>();
		self
	}

	pub fn reborrow(&mut self) -> InstanceCommands<'_, T> {
		InstanceCommands(self.0.reborrow(), PhantomData)
	}

	#[must_use]
	pub const fn cast_into<U: Kind>(self) -> InstanceCommands<'a, U>
	where
		T: CastInto<U>,
	{
		unsafe { InstanceCommands::from_entity_unchecked(self.0) }
	}
}

impl<'a, T: Kind> Deref for InstanceCommands<'a, T> {
	type Target = EntityCommands<'a>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Kind> DerefMut for InstanceCommands<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub trait GetInstanceCommands<T: Kind> {
	fn instance(&mut self, instance: Instance<T>) -> InstanceCommands<'_, T>;
}

impl<T: Kind> GetInstanceCommands<T> for Commands<'_, '_> {
	fn instance(&mut self, instance: Instance<T>) -> InstanceCommands<'_, T> {
		InstanceCommands(self.entity(instance.entity()), PhantomData)
	}
}
