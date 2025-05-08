use std::{any::TypeId, collections::HashMap};

use serde::{ Serialize, de::DeserializeOwned};
use serde_value::Value;

use super::{OptStore, OptStoreError};

#[derive(Debug, Default, Clone)]
pub struct MapStore(HashMap<(TypeId, usize), Value>);

impl MapStore {
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl OptStore for MapStore {
	fn write_value<S>(&mut self, iteration: usize, value: &S) -> Result<(), OptStoreError>
	where
		S: Serialize + 'static,
	{
		let id = TypeId::of::<S>();

		let serialized =
			serde_value::to_value(value).map_err(|e| OptStoreError::Serde(e.to_string()))?;

		self.0.insert((id, iteration), serialized);

		Ok(())
	}

	fn read_value<S>(&self, iteration: usize) -> Result<Option<S>, OptStoreError>
	where
		S: DeserializeOwned + 'static,
	{
		let id = TypeId::of::<S>();

		let Some(value) = self.0.get(&(id, iteration)).cloned() else {
			return Ok(None);
		};

		let deserialized = value
			.deserialize_into::<S>()
			.map_err(|e| OptStoreError::Serde(e.to_string()))?;

		Ok(Some(deserialized))
	}
}
