use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Datum, InternalData};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum Mutator<K = String> {
	Set(K, Datum),
	Increment(K, Datum),
	Decrement(K, Datum),
}

impl<K> Mutator<K>
where
	K: Clone + Ord,
{
	pub fn apply(&self, data: &mut InternalData<K>) {
		match self {
			Self::Set(key, value) => {
				data.insert(key.clone(), *value);
			}
			Self::Increment(key, value) => {
				if let Some(current_value) = data.get_mut(key) {
					*current_value += *value;
				}
			}
			Self::Decrement(key, value) => {
				if let Some(current_value) = data.get_mut(key) {
					*current_value -= *value;
				}
			}
		}
	}

	pub fn set(key: K, value: impl Into<Datum>) -> Self {
		Self::Set(key, value.into())
	}

	pub fn increment(key: K, value: impl Into<Datum>) -> Self {
		Self::Increment(key, value.into())
	}

	pub fn decrement(key: K, value: impl Into<Datum>) -> Self {
		Self::Decrement(key, value.into())
	}
}
