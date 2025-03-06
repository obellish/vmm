#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod instance;
pub mod prelude;

use bevy_ecs::{prelude::*, query::QueryFilter};

pub use self::instance::*;

#[macro_export]
macro_rules! kind {
	($T:ident is $U:ty) => {
		impl $crate::CastInto<$U> for $T {
			fn cast_into(instance: $crate::Instance<Self>) -> $crate::Instance<$U> {
				unsafe { instance.cast_into_unchecked() }
			}
		}
	};
}

pub struct Any;

impl Kind for Any {
	type Filter = ();
}

pub trait CastInto<T: Kind>: Kind {
	fn cast_into(instance: Instance<Self>) -> Instance<T>;
}

impl<T: Kind> CastInto<T> for T {
	fn cast_into(instance: Instance<Self>) -> Instance<Self> {
		instance
	}
}

pub trait Kind: Send + Sized + Sync + 'static {
	type Filter: QueryFilter;

	#[must_use]
	fn debug_name() -> String {
		bevy_framework_utils::get_short_name(std::any::type_name::<Self>())
	}
}

impl<T: Component> Kind for T {
	type Filter = With<T>;
}

pub trait KindBundle: Bundle {
	type Kind: Kind;
}

impl<T> KindBundle for T
where
	T: Bundle + Kind,
{
	type Kind = T;
}

pub trait SpawnInstance {
	fn spawn_instance<T: KindBundle>(&mut self, bundle: T) -> InstanceCommands<'_, T::Kind>;
}

impl SpawnInstance for Commands<'_, '_> {
	fn spawn_instance<T: KindBundle>(&mut self, bundle: T) -> InstanceCommands<'_, T::Kind> {
		let entity = self.spawn(bundle).id();
		unsafe { InstanceCommands::from_entity_unchecked(self.entity(entity)) }
	}
}

pub trait SpawnInstanceWorld {
	fn spawn_instance<T: KindBundle>(&mut self, bundle: T) -> InstanceMutItem<'_, T::Kind>
	where
		T::Kind: Component;
}

impl SpawnInstanceWorld for World {
	fn spawn_instance<T: KindBundle>(&mut self, bundle: T) -> InstanceMutItem<'_, T::Kind>
	where
		T::Kind: Component,
	{
		let entity = self.spawn(bundle).id();

		InstanceMutItem::from_entity(self, entity).unwrap()
	}
}

pub type OfKind<T> = <T as Kind>::Filter;
