use std::{
	collections::BTreeMap,
	hash::{Hash, Hasher},
};

use bevy_reflect::Reflect;

use super::Datum;

pub type InternalData = BTreeMap<String, Datum>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Reflect)]
#[repr(transparent)]
pub struct LocalState {
	pub data: InternalData,
}

impl LocalState {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			data: InternalData::new(),
		}
	}

	#[must_use]
	pub fn with_datum(mut self, key: String, value: Datum) -> Self {
		self.data.insert(key, value);
		self
	}
}

impl Hash for LocalState {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.data.len().hash(state);
		for (key, value) in &self.data {
			key.hash(state);
			value.hash(state);
		}
	}
}
