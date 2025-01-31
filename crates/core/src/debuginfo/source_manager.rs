use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::error::Error;

use thiserror::Error;

use super::{ByteIndex, FileLineColumn, Location, SourceContent, SourceFile, SourceSpan};
use crate::utils::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SourceId(u32);

impl SourceId {
	pub const UNKNOWN: Self = Self(u32::MAX);

	#[must_use]
	pub fn new(id: u32) -> Self {
		assert_ne!(
			id,
			u32::MAX,
			"u32::MAX is a reserved value for SourceId::default()/UNKNOWN"
		);

		Self::new_unchecked(id)
	}

	#[must_use]
	pub const fn new_unchecked(id: u32) -> Self {
		Self(id)
	}

	#[must_use]
	pub const fn to_usize(self) -> usize {
		self.0 as usize
	}

	#[must_use]
	pub const fn to_u32(self) -> u32 {
		self.0
	}

	#[must_use]
	pub const fn is_unknown(self) -> bool {
		self.0 == u32::MAX
	}
}

impl Default for SourceId {
	fn default() -> Self {
		Self::UNKNOWN
	}
}

impl TryFrom<usize> for SourceId {
	type Error = TryFromIntError;

	fn try_from(value: usize) -> Result<Self, Self::Error> {
		let value = u32::try_from(value)?;

		if value < u32::MAX {
			Ok(Self::new_unchecked(value))
		} else {
			Err(TryFromIntError(()))
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("out of range integral type conversion attempted")]
pub struct TryFromIntError(());

impl From<core::num::TryFromIntError> for TryFromIntError {
	fn from(_: core::num::TryFromIntError) -> Self {
		Self(())
	}
}

#[derive(Default)]
pub struct DefaultSourceManager(RwLock<DefaultSourceManagerImpl>);

impl Clone for DefaultSourceManager {
	fn clone(&self) -> Self {
		let manager = self.0.read();
		Self(RwLock::new(manager.clone()))
	}
}

impl SourceManager for DefaultSourceManager {
	fn load_from_raw_parts(&self, name: Arc<str>, content: SourceContent) -> Arc<SourceFile> {
		let mut manager = self.0.write();
		manager.insert(name, content)
	}

	fn get(&self, id: SourceId) -> Result<Arc<SourceFile>, SourceManagerError> {
		let manager = self.0.read();
		manager.get(id)
	}

	fn find(&self, name: &str) -> Option<SourceId> {
		let manager = self.0.read();
		manager.find(name)
	}

	fn file_line_column_to_span(&self, loc: FileLineColumn) -> Option<SourceSpan> {
		let manager = self.0.read();
		manager.file_line_column_to_span(loc)
	}

	fn file_line_column(&self, span: SourceSpan) -> Result<FileLineColumn, SourceManagerError> {
		let manager = self.0.read();
		manager.file_line_column(span)
	}

	fn location_to_span(&self, loc: Location) -> Option<SourceSpan> {
		let manager = self.0.read();
		manager.location_to_span(loc)
	}

	fn location(&self, span: SourceSpan) -> Result<Location, SourceManagerError> {
		let manager = self.0.read();
		manager.location(span)
	}

	fn source(&self, id: SourceId) -> Result<&str, SourceManagerError> {
		let manager = self.0.read();
		let ptr = manager
			.files
			.get(id.to_usize())
			.ok_or(SourceManagerError::InvalidSourceId)
			.map(|file| core::ptr::from_ref::<str>(file.as_str()))?;

		drop(manager);

		Ok(unsafe { &*ptr })
	}

	fn source_slice(&self, span: SourceSpan) -> Result<&str, SourceManagerError> {
		self.source(span.source_id())?
			.get(span.into_slice_index())
			.ok_or(SourceManagerError::InvalidBounds)
	}

	fn get_by_path(&self, path: &str) -> Option<Arc<SourceFile>> {
		let manager = self.0.read();
		manager.get_by_path(path)
	}
}

#[derive(Default, Clone)]
struct DefaultSourceManagerImpl {
	files: Vec<Arc<SourceFile>>,
	names: BTreeMap<Arc<str>, SourceId>,
}

impl DefaultSourceManagerImpl {
	fn insert(&mut self, name: Arc<str>, content: SourceContent) -> Arc<SourceFile> {
		if let Some(file) = self.names.get(&name).copied().and_then(|id| {
			let file = &self.files[id.to_usize()];
			if file.as_str() == content.as_str() {
				Some(Arc::clone(file))
			} else {
				None
			}
		}) {
			return file;
		}

		let id = SourceId::try_from(self.files.len())
			.expect("system limit: source manager has exhausted it's supply of source ids");
		let file = Arc::new(SourceFile::from_raw_parts(id, content));
		self.files.push(Arc::clone(&file));
		file
	}

	fn get(&self, id: SourceId) -> Result<Arc<SourceFile>, SourceManagerError> {
		self.files
			.get(id.to_usize())
			.cloned()
			.ok_or(SourceManagerError::InvalidSourceId)
	}

	fn get_by_path(&self, path: &str) -> Option<Arc<SourceFile>> {
		self.find(path).and_then(|id| self.get(id).ok())
	}

	fn find(&self, name: &str) -> Option<SourceId> {
		self.names.get(name).copied()
	}

	fn file_line_column_to_span(&self, loc: FileLineColumn) -> Option<SourceSpan> {
		let file = self
			.names
			.get(&loc.path)
			.copied()
			.and_then(|id| self.files.get(id.to_usize()))?;

		file.line_column_to_span(loc.line, loc.column)
	}

	fn file_line_column(&self, span: SourceSpan) -> Result<FileLineColumn, SourceManagerError> {
		self.files
			.get(span.source_id().to_usize())
			.ok_or(SourceManagerError::InvalidSourceId)
			.map(|file| file.location(span))
	}

	fn location_to_span(&self, loc: Location) -> Option<SourceSpan> {
		let file = self
			.names
			.get(&loc.path)
			.copied()
			.and_then(|id| self.files.get(id.to_usize()))?;

		let max_len = ByteIndex::from(file.as_str().len() as u32);
		if loc.start >= max_len || loc.end > max_len {
			return None;
		}

		Some(SourceSpan::new(file.id(), loc.start..loc.end))
	}

	fn location(&self, span: SourceSpan) -> Result<Location, SourceManagerError> {
		self.files
			.get(span.source_id().to_usize())
			.ok_or(SourceManagerError::InvalidSourceId)
			.map(|file| Location::new(file.name(), span.start(), span.end()))
	}
}

#[derive(Debug, Error)]
pub enum SourceManagerError {
	#[error("attempted to use an invalid source id")]
	InvalidSourceId,
	#[error("attempted to read content out of bounds")]
	InvalidBounds,
	#[error("{error_message}")]
	Custom {
		error_message: Box<str>,
		#[source]
		source: Option<Box<dyn Error + Send + Sync + 'static>>,
	},
}

impl SourceManagerError {
	#[must_use]
	pub fn custom(message: String) -> Self {
		Self::Custom {
			error_message: message.into_boxed_str(),
			source: None,
		}
	}

	pub fn custom_with_source(message: String, source: impl Error + Send + Sync + 'static) -> Self {
		Self::Custom {
			error_message: message.into_boxed_str(),
			source: Some(Box::new(source)),
		}
	}
}

pub trait SourceManager {
	fn load_from_raw_parts(&self, name: Arc<str>, content: SourceContent) -> Arc<SourceFile>;

	fn get(&self, id: SourceId) -> Result<Arc<SourceFile>, SourceManagerError>;

	fn find(&self, name: &str) -> Option<SourceId>;

	fn file_line_column_to_span(&self, loc: FileLineColumn) -> Option<SourceSpan>;

	fn file_line_column(&self, span: SourceSpan) -> Result<FileLineColumn, SourceManagerError>;

	fn location_to_span(&self, loc: Location) -> Option<SourceSpan>;

	fn location(&self, span: SourceSpan) -> Result<Location, SourceManagerError>;

	fn source(&self, id: SourceId) -> Result<&str, SourceManagerError>;

	fn source_slice(&self, span: SourceSpan) -> Result<&str, SourceManagerError>;

	fn is_manager_of(&self, file: &SourceFile) -> bool {
		self.get(file.id())
			.is_ok_and(|found| core::ptr::addr_eq(Arc::as_ptr(&found), file))
	}

	fn copy_into(&self, file: &SourceFile) -> Arc<SourceFile> {
		if let Ok(found) = self.get(file.id()) {
			if core::ptr::addr_eq(Arc::as_ptr(&found), file) {
				return found;
			}
		}

		self.load_from_raw_parts(file.name(), file.content().clone())
	}

	fn load(&self, name: &str, content: String) -> Arc<SourceFile> {
		let name = Arc::from(String::from(name));
		let content = SourceContent::new(Arc::clone(&name), content.into_boxed_str());
		self.load_from_raw_parts(name, content)
	}

	fn get_by_path(&self, path: &str) -> Option<Arc<SourceFile>> {
		self.find(path).and_then(|id| self.get(id).ok())
	}
}

impl<T> SourceManager for Arc<T>
where
	T: ?Sized + SourceManager,
{
	fn load_from_raw_parts(&self, name: Arc<str>, content: SourceContent) -> Arc<SourceFile> {
		(**self).load_from_raw_parts(name, content)
	}

	fn get(&self, id: SourceId) -> Result<Arc<SourceFile>, SourceManagerError> {
		(**self).get(id)
	}

	fn find(&self, name: &str) -> Option<SourceId> {
		(**self).find(name)
	}

	fn file_line_column_to_span(&self, loc: FileLineColumn) -> Option<SourceSpan> {
		(**self).file_line_column_to_span(loc)
	}

	fn file_line_column(&self, span: SourceSpan) -> Result<FileLineColumn, SourceManagerError> {
		(**self).file_line_column(span)
	}

	fn location(&self, span: SourceSpan) -> Result<Location, SourceManagerError> {
		(**self).location(span)
	}

	fn location_to_span(&self, loc: Location) -> Option<SourceSpan> {
		(**self).location_to_span(loc)
	}

	fn source(&self, id: SourceId) -> Result<&str, SourceManagerError> {
		(**self).source(id)
	}

	fn source_slice(&self, span: SourceSpan) -> Result<&str, SourceManagerError> {
		(**self).source_slice(span)
	}
}

#[cfg(feature = "std")]
pub trait SourceManagerExt: SourceManager {
	fn load_file(&self, path: &std::path::Path) -> Result<Arc<SourceFile>, SourceManagerError> {
		let name = path.to_string_lossy();
		if let Some(existing) = self.get_by_path(name.as_ref()) {
			return Ok(existing);
		}

		let name = Arc::from(name.into_owned());
		let content = std::fs::read_to_string(path)
			.map(|s| SourceContent::new(Arc::clone(&name), s.into_boxed_str()))
			.map_err(|source| {
				SourceManagerError::custom_with_source(
					format!("failed to load file at `{}`", path.display()),
					source,
				)
			})?;

		Ok(self.load_from_raw_parts(name, content))
	}
}

#[cfg(feature = "std")]
impl<T> SourceManagerExt for T where T: ?Sized + SourceManager {}

#[cfg(test)]
mod tests {
	use core::error::Error;

	use static_assertions::assert_impl_all;

	use super::SourceManagerError;

	assert_impl_all!(SourceManagerError: Error, Send, Sync);
}
