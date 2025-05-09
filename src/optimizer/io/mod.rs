mod map;
mod noop;
mod ron;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::Error as IoError,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use self::{map::*, noop::*, ron::*};
use super::AnalysisOutput;
use crate::{Instruction, Program};

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
	/// Write a raw, serializable value to the store.
	fn write_value<S>(&mut self, iteration: usize, value: &S) -> Result<(), OptStoreError>
	where
		S: Serialize + 'static;

	/// Read and deserialize a value from the store.
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

	fn write_program(&mut self, iteration: usize, program: &Program) -> Result<(), OptStoreError> {
		self.write_value(iteration, program)?;

		self.write_value(
			iteration,
			&program.into_iter().collect::<RawProgram>(),
		)?;

		Ok(())
	}

	fn read_program(&self, iteration: usize) -> Result<Option<Program>, OptStoreError> {
		self.read_value(iteration)
	}
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub(super) struct RawProgram(String);

impl Debug for RawProgram {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for RawProgram {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl FromIterator<Instruction> for RawProgram {
	fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
		Self(iter.into_iter().map(|i| i.to_string()).collect())
	}
}

impl<'a> FromIterator<&'a Instruction> for RawProgram {
	fn from_iter<T: IntoIterator<Item = &'a Instruction>>(iter: T) -> Self {
		Self(iter.into_iter().map(ToString::to_string).collect())
	}
}
