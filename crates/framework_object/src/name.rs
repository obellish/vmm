use bevy_framework_kind::prelude::*;

use super::{Object, ObjectInstance as _, ObjectRef};

pub trait ObjectName {
	fn name(&self) -> Option<&str>;
}

impl<T: Kind> ObjectName for Object<'_, '_, '_, T> {
	fn name(&self) -> Option<&str> {
		self.name.get(self.entity()).ok().map(bevy_core::Name::as_str)
	}
}

impl<T: Kind> ObjectName for ObjectRef<'_, '_, '_, T> {
	fn name(&self) -> Option<&str> {
		self.1.name()
	}
}
