#[cfg(feature = "output")]
mod output;

use std::{
	any::TypeId,
	collections::HashMap,
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::Error as IoError,
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use vmm_ir::Instruction;

#[cfg(feature = "output")]
pub use self::output::*;

#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct HashMetadataStore {
	inner: HashMap<(TypeId, usize), Value>,
}

impl HashMetadataStore {
	#[must_use]
	pub fn new() -> Self {
		Self {
			inner: HashMap::new(),
		}
	}
}

impl MetadataStore for HashMetadataStore {
	fn insert<S>(&mut self, iteration: usize, value: &S) -> Result<(), MetadataStoreError>
	where
		S: Serialize + 'static,
	{
		let id = TypeId::of::<S>();

		let serialized = serde_value::to_value(value)?;

		self.inner.insert((id, iteration), serialized);

		Ok(())
	}

	fn get<S>(&self, iteration: usize) -> Result<Option<S>, MetadataStoreError>
	where
		S: for<'de> Deserialize<'de> + 'static,
	{
		let id = TypeId::of::<S>();

		let Some(value) = self.inner.get(&(id, iteration)).cloned() else {
			return Ok(None);
		};

		let deserialized = value.deserialize_into()?;

		Ok(Some(deserialized))
	}
}

#[derive(Debug)]
pub enum MetadataStoreError {
	Serde(String),
	Io(IoError),
}

impl Display for MetadataStoreError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Serde(s) => {
				f.write_str("error during (de)serialization: ")?;
				f.write_str(s)?;
			}
			Self::Io(e) => Display::fmt(&e, f)?,
		}

		Ok(())
	}
}

impl Eq for MetadataStoreError {}

impl StdError for MetadataStoreError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Serde(_) => None,
			Self::Io(e) => Some(e),
		}
	}
}

impl From<IoError> for MetadataStoreError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}

#[cfg(feature = "output")]
impl From<::ron::Error> for MetadataStoreError {
	fn from(value: ::ron::Error) -> Self {
		Self::Serde(value.to_string())
	}
}

impl From<serde_value::DeserializerError> for MetadataStoreError {
	fn from(value: serde_value::DeserializerError) -> Self {
		Self::Serde(value.to_string())
	}
}

impl From<serde_value::SerializerError> for MetadataStoreError {
	fn from(value: serde_value::SerializerError) -> Self {
		Self::Serde(value.to_string())
	}
}

impl PartialEq for MetadataStoreError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Serde(l), Self::Serde(r)) => l == r,
			(Self::Io(l), Self::Io(r)) => l.kind() == r.kind(),
			_ => false,
		}
	}
}

pub trait MetadataStore {
	fn get<S>(&self, iteration: usize) -> Result<Option<S>, MetadataStoreError>
	where
		S: for<'de> Deserialize<'de> + 'static;

	fn insert<S>(&mut self, iteration: usize, value: &S) -> Result<(), MetadataStoreError>
	where
		S: Serialize + 'static;

	fn insert_program_snapshot(
		&mut self,
		iteration: usize,
		program: &vmm_program::Program,
	) -> Result<(), MetadataStoreError> {
		self.insert(iteration, &program.into_iter().collect::<RawProgram>())?;
		let program = Program(program.into_iter().cloned().collect());

		self.insert(iteration, &program)?;

		Ok(())
	}

	fn get_program_snapshot(
		&self,
		iteration: usize,
	) -> Result<Option<vmm_program::Program>, MetadataStoreError> {
		let Some(program) = self.get::<Program>(iteration)? else {
			return Ok(None);
		};

		Ok(Some(program.0.into_par_iter().collect()))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct Program(Vec<Instruction>);

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
