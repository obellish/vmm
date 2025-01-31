use alloc::sync::Arc;
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::Range,
};

use super::ByteIndex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
	pub path: Arc<str>,
	pub start: ByteIndex,
	pub end: ByteIndex,
}

impl Location {
	#[must_use]
	pub const fn new(path: Arc<str>, start: ByteIndex, end: ByteIndex) -> Self {
		Self { path, start, end }
	}

	#[must_use]
	pub fn path(&self) -> Arc<str> {
		Arc::clone(&self.path)
	}

	#[must_use]
	pub const fn range(&self) -> Range<ByteIndex> {
		self.start..self.end
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileLineColumn {
	pub path: Arc<str>,
	pub line: u32,
	pub column: u32,
}

impl FileLineColumn {
	#[must_use]
	pub const fn new(path: Arc<str>, line: u32, column: u32) -> Self {
		Self { path, line, column }
	}

	#[must_use]
	pub fn path(&self) -> Arc<str> {
		Arc::clone(&self.path)
	}

	#[must_use]
	pub const fn line(&self) -> u32 {
		self.line
	}

	#[must_use]
	pub const fn column(&self) -> u32 {
		self.column
	}

	pub fn move_column(&mut self, offset: u32) {
		self.column += offset;
	}
}

impl Display for FileLineColumn {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('[')?;
		Display::fmt(&self.path, f)?;
		f.write_char('@')?;
		Display::fmt(&self.line, f)?;
		f.write_char(':')?;
		Display::fmt(&self.column, f)?;
		f.write_char(']')
	}
}
