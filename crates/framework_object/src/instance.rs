use bevy_ecs::prelude::*;
use bevy_framework_kind::{Any, prelude::*};

use super::{Object, ObjectRef};

pub trait ObjectInstance<T: Kind = Any> {
	fn instance(&self) -> Instance<T>;

	fn entity(&self) -> Entity {
		self.instance().entity()
	}
}

impl<T: Kind> ObjectInstance<T> for Object<'_, '_, '_, T> {
	fn instance(&self) -> Instance<T> {
		self.instance
	}
}

impl<T: Kind> ObjectInstance<T> for ObjectRef<'_, '_, '_, T> {
	fn instance(&self) -> Instance<T> {
		self.1.instance()
	}
}
