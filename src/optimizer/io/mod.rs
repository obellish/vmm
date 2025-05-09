mod map;
mod noop;
mod ron;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IoError,
};

use serde::{Serialize, de::DeserializeOwned};

pub use self::{map::*, noop::*, ron::*};
use super::AnalysisOutput;

#[derive(Debug)]
pub enum OptStoreError {
	Serde(String),
	Io(IoError),
}

impl Display for OptStoreError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Serde(s) => {
				f.write_str("error with serialization: ")?;
				f.write_str(s)
			}
			Self::Io(e) => Display::fmt(&e, f),
		}
	}
}

impl Eq for OptStoreError {}

impl StdError for OptStoreError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Serde(_) => None,
			Self::Io(e) => Some(e),
		}
	}
}

impl From<IoError> for OptStoreError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}

impl PartialEq for OptStoreError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Serde(l), Self::Serde(r)) => l == r,
			(Self::Io(l), Self::Io(r)) => l.kind() == r.kind(),
			_ => false,
		}
	}
}

/// Intermediate Optimizations store (aka results of things between passes).
pub trait OptStore {
	/// Write a raw, serializable value to the store
	fn write_value<S>(&mut self, iteration: usize, value: &S) -> Result<(), OptStoreError>
	where
		S: Serialize + 'static;

	fn read_value<S>(&self, iteration: usize) -> Result<Option<S>, OptStoreError>
	where
		S: DeserializeOwned + 'static;

	fn write_analysis_output(
		&mut self,
		iteration: usize,
		value: &AnalysisOutput,
	) -> Result<(), OptStoreError> {
		self.write_value(iteration, value)?;

		Ok(())
	}

	fn read_analysis_output(
		&self,
		iteration: usize,
	) -> Result<Option<AnalysisOutput>, OptStoreError> {
		self.read_value(iteration)
	}
}
