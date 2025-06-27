use super::MetadataStore;

#[derive(Debug, Default)]
pub struct NoopStore;

impl NoopStore {
	#[must_use]
	pub const fn new() -> Self {
		Self
	}
}

impl MetadataStore for NoopStore {
	fn get<S>(&self, _: usize) -> Result<Option<S>, super::MetadataStoreError>
	where
		S: for<'de> serde::Deserialize<'de> + 'static,
	{
		Ok(None)
	}

	fn insert<S>(&mut self, _: usize, _: &S) -> Result<(), super::MetadataStoreError>
	where
		S: serde::Serialize + 'static,
	{
		Ok(())
	}
}
