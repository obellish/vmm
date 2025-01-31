mod location;
mod source_file;
mod source_manager;
mod span;

#[cfg(feature = "std")]
pub use self::source_manager::SourceManagerExt;
pub use self::{
	location::{FileLineColumn, Location},
	source_file::{
		ByteIndex, ByteOffset, ColumnIndex, LineIndex, SourceContent, SourceFile, SourceFileRef,
	},
	source_manager::{
		DefaultSourceManager, SourceId, SourceManager, SourceManagerError, TryFromIntError,
	},
	span::{SourceSpan, Span, Spanned},
};
