use std::{fs, path::PathBuf};

use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use tap::prelude::*;
use vmm_type_name::ShortName;

use super::{MetadataStore, MetadataStoreError};

#[derive(Debug, Clone)]
pub struct OutputMetadataStore<T> {
	inner: T,
	folder_path: PathBuf,
}

impl<T> OutputMetadataStore<T> {
	#[inline]
	pub fn new(inner: T, folder_path: PathBuf) -> Result<Self, MetadataStoreError> {
		let this = Self { inner, folder_path };

		this.ensure_folder()?;

		Ok(this)
	}

	#[inline]
	fn ensure_folder(&self) -> Result<(), MetadataStoreError> {
		assert!(self.folder_path.is_dir());

		fs::create_dir_all(&self.folder_path)?;

		Ok(())
	}
}

impl<T: MetadataStore> MetadataStore for OutputMetadataStore<T> {
	#[inline]
	fn get<S>(&self, iteration: usize) -> Result<Option<S>, MetadataStoreError>
	where
		S: for<'de> Deserialize<'de> + 'static,
	{
		self.inner.get(iteration)
	}

	#[inline]
	fn insert<S>(&mut self, iteration: usize, value: &S) -> Result<(), MetadataStoreError>
	where
		S: Serialize + 'static,
	{
		self.inner.insert(iteration, value)?;

		let serialized = {
			let mut output = String::new();
			let mut serializer = ron::Serializer::with_options(
				&mut output,
				Some(config()),
				&ron::Options::default(),
			)?;
			let serializer = serde_stack::Serializer::new(&mut serializer);
			value.serialize(serializer)?;

			output
		};

		self.folder_path
			.join(format!(
				"{}-{iteration}.ron",
				ShortName(std::any::type_name::<S>())
			))
			.pipe(|value| fs::write(value, serialized))?;

		Ok(())
	}
}

fn config() -> PrettyConfig {
	PrettyConfig::new().separate_tuple_members(true)
}
