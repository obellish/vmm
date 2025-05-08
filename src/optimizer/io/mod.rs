mod map;
mod noop;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_array::Array;

pub use self::noop::*;
use crate::{CellState, TAPE_SIZE};

#[derive(Debug, PartialEq, Eq)]
pub enum OptStoreError {
	Serde(String),
}

impl Display for OptStoreError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Serde(s) => {
				f.write_str("error with serialization: ")?;
				f.write_str(s)
			}
		}
	}
}

impl StdError for OptStoreError {}

/// Intermediate Optimizations store (aka results of things between passes).
pub trait OptStore {
	/// Write a raw, serializable value to the store
	fn write_value<S>(&mut self, iteration: usize, value: &S) -> Result<(), OptStoreError>
	where
		S: Serialize + 'static;

	fn read_value< S>(&self, iteration: usize) -> Result<Option<S>, OptStoreError>
	where
		S: DeserializeOwned + 'static;

	fn write_analysis_output(
		&mut self,
		iteration: usize,
		value: &[CellState; TAPE_SIZE],
	) -> Result<(), OptStoreError> {
		let a = Array(*value);

		self.write_value(iteration, &a)?;

		Ok(())
	}
}
