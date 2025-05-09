use std::{fs, path::PathBuf, sync::LazyLock};

use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Serialize, de::DeserializeOwned};
use vmm_type_name::ShortName;

use super::{OptStore, OptStoreError};

static CONFIG: LazyLock<PrettyConfig> =
	LazyLock::new(|| PrettyConfig::new().separate_tuple_members(true));

#[derive(Debug, Clone)]
pub struct RonWrapperStore<T> {
	inner: T,
	folder_path: PathBuf,
}

impl<T> RonWrapperStore<T> {
	pub fn new(inner: T, folder_path: PathBuf) -> Result<Self, OptStoreError> {
		let this = Self { inner, folder_path };

		this.ensure_folder()?;

		Ok(this)
	}

	fn ensure_folder(&self) -> Result<(), OptStoreError> {
		assert!(self.folder_path.is_dir());

		fs::create_dir_all(&self.folder_path)?;

		Ok(())
	}
}

impl<T: OptStore> OptStore for RonWrapperStore<T> {
	fn write_value<S>(&mut self, iteration: usize, value: &S) -> Result<(), OptStoreError>
	where
		S: Serialize + 'static,
	{
		self.inner.write_value(iteration, value)?;

		let serialized = to_string_pretty(value, CONFIG.clone())
			.map_err(|e| OptStoreError::Serde(e.to_string()))?;

		let file_name = self.folder_path.join(format!(
			"{}-{iteration}.ron",
			ShortName(std::any::type_name::<S>())
		));

		fs::write(file_name, serialized)?;

		Ok(())
	}

	fn read_value<S>(&self, iteration: usize) -> Result<Option<S>, OptStoreError>
	where
		S: DeserializeOwned + 'static,
	{
		self.inner.read_value(iteration)
	}
}
