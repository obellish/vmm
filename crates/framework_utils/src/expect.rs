use std::marker::PhantomData;

use bevy_ecs::{
	archetype::Archetype,
	component::{ComponentId, Components, Tick},
	prelude::*,
	query::{FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery},
	storage::{Table, TableRow},
	world::unsafe_world_cell::UnsafeWorldCell,
};

#[repr(transparent)]
pub struct Expect<T: QueryData>(PhantomData<T>);

unsafe impl<T: QueryData> QueryData for Expect<T> {
	type ReadOnly = Expect<T::ReadOnly>;
}

unsafe impl<T: ReadOnlyQueryData> ReadOnlyQueryData for Expect<T> {}

unsafe impl<T: QueryData> WorldQuery for Expect<T> {
	type Fetch<'w> = ExpectFetch<'w, T>;
	type Item<'w> = T::Item<'w>;
	type State = T::State;

	const IS_DENSE: bool = T::IS_DENSE;

	fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
		T::shrink(item)
	}

	fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
		ExpectFetch {
			fetch: T::shrink_fetch(fetch.fetch),
			matches: fetch.matches,
		}
	}

	unsafe fn init_fetch<'w>(
		world: UnsafeWorldCell<'w>,
		state: &Self::State,
		last_run: Tick,
		this_run: Tick,
	) -> Self::Fetch<'w> {
		ExpectFetch {
			fetch: unsafe { T::init_fetch(world, state, last_run, this_run) },
			matches: false,
		}
	}

	unsafe fn set_archetype<'w>(
		fetch: &mut Self::Fetch<'w>,
		state: &Self::State,
		archetype: &'w Archetype,
		table: &'w Table,
	) {
		fetch.matches = T::matches_component_set(state, &|id| archetype.contains(id));
		if fetch.matches {
			unsafe { T::set_archetype(&mut fetch.fetch, state, archetype, table) };
		}
	}

	unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
		fetch.matches = T::matches_component_set(state, &|id| table.has_column(id));
		if fetch.matches {
			unsafe { T::set_table(&mut fetch.fetch, state, table) };
		}
	}

	unsafe fn fetch<'w>(
		fetch: &mut Self::Fetch<'w>,
		entity: Entity,
		table_row: TableRow,
	) -> Self::Item<'w> {
		let item = fetch
			.matches
			.then(|| unsafe { T::fetch(&mut fetch.fetch, entity, table_row) });

		let Some(item) = item else {
			panic!(
				"expected query of type `{}` does not match entity {:?}",
				std::any::type_name::<T>(),
				entity
			);
		};

		item
	}

	fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
		let mut intermediate = access.clone();
		T::update_component_access(state, &mut intermediate);
		access.extend_access(&intermediate);
	}

	fn get_state(components: &Components) -> Option<Self::State> {
		T::get_state(components)
	}

	fn init_state(world: &mut World) -> Self::State {
		T::init_state(world)
	}

	fn matches_component_set(
		_state: &Self::State,
		_set_contains_id: &impl Fn(ComponentId) -> bool,
	) -> bool {
		true
	}
}

pub struct ExpectFetch<'w, T: WorldQuery> {
	fetch: T::Fetch<'w>,
	matches: bool,
}

impl<T: WorldQuery> Clone for ExpectFetch<'_, T> {
	fn clone(&self) -> Self {
		Self {
			fetch: self.fetch.clone(),
			matches: self.matches,
		}
	}
}
