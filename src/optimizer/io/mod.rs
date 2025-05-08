mod noop;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use serde::{Deserialize, Serialize};
use serde_array::Array;

pub use self::noop::*;
use crate::{CellState, TAPE_SIZE};

#[derive(Debug, PartialEq, Eq)]
pub enum OptStoreError {}

impl Display for OptStoreError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for OptStoreError {}

/// Intermediate Optimizations store (aka results of things between passes).
pub trait OptStore {
	/// Write a raw, serializable value to the store
	fn write_value<S: Serialize>(&self, iteration: usize, value: &S) -> Result<(), OptStoreError>;

	fn read_value<'de, S>(&self, iteration: usize) -> Option<S>
	where
		S: Deserialize<'de>;

	fn write_analysis_output(
		&self,
		iteration: usize,
		value: &[CellState; TAPE_SIZE],
	) -> Result<(), OptStoreError> {
		let a = Array(*value);

		self.write_value(iteration, &a)?;

		Ok(())
	}
}
