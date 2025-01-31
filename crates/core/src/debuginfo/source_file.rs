use alloc::{boxed::Box, string::String, sync::Arc};
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	num::NonZeroU32,
	ops::{Add, AddAssign, Deref, Range, Sub, SubAssign},
};

use super::{FileLineColumn, SourceId, SourceSpan};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteIndex(u32);

impl ByteIndex {
	#[must_use]
	pub const fn new(index: u32) -> Self {
		Self(index)
	}

	#[must_use]
	pub const fn to_usize(self) -> usize {
		self.0 as usize
	}

	#[must_use]
	pub const fn to_u32(self) -> u32 {
		self.0
	}
}

impl Add<ByteOffset> for ByteIndex {
	type Output = Self;

	fn add(self, rhs: ByteOffset) -> Self::Output {
		Self::new((i64::from(self.0) + rhs.0) as u32)
	}
}

impl Add<u32> for ByteIndex {
	type Output = Self;

	fn add(self, rhs: u32) -> Self::Output {
		Self::new(self.0 + rhs)
	}
}

impl AddAssign<ByteOffset> for ByteIndex {
	fn add_assign(&mut self, rhs: ByteOffset) {
		*self = *self + rhs;
	}
}

impl AddAssign<u32> for ByteIndex {
	fn add_assign(&mut self, rhs: u32) {
		*self = *self + rhs;
	}
}

impl From<u32> for ByteIndex {
	fn from(value: u32) -> Self {
		Self::new(value)
	}
}

impl From<ByteIndex> for u32 {
	fn from(value: ByteIndex) -> Self {
		value.to_u32()
	}
}

impl Sub<ByteOffset> for ByteIndex {
	type Output = Self;

	fn sub(self, rhs: ByteOffset) -> Self::Output {
		Self::new((i64::from(self.0) - rhs.0) as u32)
	}
}

impl Sub<u32> for ByteIndex {
	type Output = Self;

	fn sub(self, rhs: u32) -> Self::Output {
		Self::new(self.0 - rhs)
	}
}

impl SubAssign<ByteOffset> for ByteIndex {
	fn sub_assign(&mut self, rhs: ByteOffset) {
		*self = *self - rhs;
	}
}

impl SubAssign<u32> for ByteIndex {
	fn sub_assign(&mut self, rhs: u32) {
		*self = *self - rhs;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteOffset(i64);

impl ByteOffset {
	#[must_use]
	pub const fn from_char_len(c: char) -> Self {
		Self(c.len_utf8() as i64)
	}

	#[must_use]
	pub const fn from_str_len(s: &str) -> Self {
		Self(s.len() as i64)
	}
}

impl Add for ByteOffset {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign for ByteOffset {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl Sub for ByteOffset {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign for ByteOffset {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ColumnIndex(u32);

impl ColumnIndex {
	#[must_use]
	pub const fn number(self) -> NonZeroU32 {
		unsafe { NonZeroU32::new_unchecked(self.0 + 1) }
	}

	#[must_use]
	pub const fn to_usize(self) -> usize {
		self.0 as usize
	}
}

impl From<u32> for ColumnIndex {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

impl From<ColumnIndex> for u32 {
	fn from(value: ColumnIndex) -> Self {
		value.0
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct LineIndex(u32);

impl LineIndex {
	#[must_use]
	pub const fn number(self) -> NonZeroU32 {
		unsafe { NonZeroU32::new_unchecked(self.0 + 1) }
	}

	#[must_use]
	pub const fn to_usize(self) -> usize {
		self.0 as usize
	}

	pub fn checked_add(self, offset: u32) -> Option<Self> {
		self.0.checked_add(offset).map(Self)
	}

	pub fn checked_sub(self, offset: u32) -> Option<Self> {
		self.0.checked_sub(offset).map(Self)
	}

	#[must_use]
	pub const fn saturating_add(self, offset: u32) -> Self {
		Self(self.0.saturating_add(offset))
	}

	#[must_use]
	pub const fn saturating_sub(self, offset: u32) -> Self {
		Self(self.0.saturating_sub(offset))
	}
}

impl Add<u32> for LineIndex {
	type Output = Self;

	fn add(self, rhs: u32) -> Self::Output {
		Self(self.0 + rhs)
	}
}

impl From<u32> for LineIndex {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

impl From<LineIndex> for u32 {
	fn from(value: LineIndex) -> Self {
		value.0
	}
}

#[derive(Clone)]
pub struct SourceContent {
	path: Arc<str>,
	content: Box<str>,
	line_starts: Box<[ByteIndex]>,
}

impl SourceContent {
	#[must_use]
	pub fn new(path: Arc<str>, content: Box<str>) -> Self {
		let bytes = content.as_bytes();

		assert!(
			bytes.len() < u32::MAX as usize,
			"unsupported file size: current maximum supported length in bytes is 2^32"
		);

		let line_starts = core::iter::once(ByteIndex(0))
			.chain(
				memchr::memchr_iter(b'\n', content.as_bytes()).filter_map(|mut offset| {
					let mut preceding_escapes = 0;
					let line_start = offset + 1;
					while let Some(prev_offset) = offset.checked_sub(1) {
						if matches!(bytes[prev_offset], b'\\') {
							offset = prev_offset;
							preceding_escapes += 1;
							continue;
						}
						break;
					}

					let is_escaped = preceding_escapes > 0 && !matches!(preceding_escapes % 2, 0);
					if is_escaped {
						None
					} else {
						Some(ByteIndex(line_start as u32))
					}
				}),
			)
			.collect();

		Self {
			path,
			content,
			line_starts,
		}
	}

	#[must_use]
	pub fn name(&self) -> Arc<str> {
		Arc::clone(&self.path)
	}

	#[cfg(feature = "std")]
	#[must_use]
	pub fn path(&self) -> &std::path::Path {
		std::path::Path::new(self.path.as_ref())
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.content
	}

	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		self.content.as_bytes()
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.content.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.content.is_empty()
	}

	#[must_use]
	pub fn source_range(&self) -> Range<ByteIndex> {
		ByteIndex(0)..ByteIndex(self.len() as u32)
	}

	pub fn source_slice(&self, span: impl Into<Range<usize>>) -> Option<&str> {
		self.as_str().get(span.into())
	}

	#[must_use]
	pub fn line_start(&self, line_index: LineIndex) -> Option<ByteIndex> {
		self.line_starts.get(line_index.to_usize()).copied()
	}

	#[must_use]
	pub fn last_line_index(&self) -> LineIndex {
		LineIndex(self.line_starts.len() as u32)
	}

	#[must_use]
	pub fn line_range(&self, line_index: LineIndex) -> Option<Range<ByteIndex>> {
		let line_start = self.line_start(line_index)?;
		self.line_start(line_index + 1).map_or_else(
			|| Some(line_start..ByteIndex(self.len() as u32)),
			|line_end| Some(line_start..line_end),
		)
	}

	#[must_use]
	pub fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
		match self.line_starts.binary_search(&byte_index) {
			Ok(line) => LineIndex(line as u32),
			Err(next_line) => LineIndex(next_line as u32 - 1),
		}
	}

	#[must_use]
	pub fn line_column_to_offset(
		&self,
		line_index: LineIndex,
		column_index: ColumnIndex,
	) -> Option<ByteIndex> {
		let column_index = column_index.to_usize();
		let line_span = self.line_range(line_index)?;
		let line_src = self
			.content
			.get(line_span.start.to_usize()..line_span.end.to_usize())
			.expect("invalid line boundaries: invalid utf-8");
		if line_src.len() < column_index {
			return None;
		}

		let (pre, _) = line_src.split_at(column_index);
		let start = line_span.start;
		Some(start + ByteOffset::from_str_len(pre))
	}

	#[must_use]
	pub fn location(&self, byte_index: ByteIndex) -> Option<FileLineColumn> {
		let line_index = self.line_index(byte_index);
		let line_start_index = self.line_start(line_index)?;
		let line_src = self
			.content
			.get(line_start_index.to_usize()..byte_index.to_usize())?;
		let column_index = ColumnIndex::from(line_src.chars().count() as u32);
		Some(FileLineColumn {
			path: self.name(),
			line: line_index.number().get(),
			column: column_index.number().get(),
		})
	}
}

impl Debug for SourceContent {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("SourceContent")
			.field("path", &self.path)
			.field("size_in_bytes", &self.content.len())
			.field("line_count", &self.line_starts.len())
			.field("content", &self.content)
			.finish()
	}
}

impl Eq for SourceContent {}

impl Hash for SourceContent {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.path.hash(state);
		self.content.hash(state);
	}
}

impl Ord for SourceContent {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.path
			.cmp(&other.path)
			.then_with(|| self.content.cmp(&other.content))
	}
}

impl PartialEq for SourceContent {
	fn eq(&self, other: &Self) -> bool {
		self.path == other.path && self.content == other.content
	}
}

impl PartialOrd for SourceContent {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceFile {
	id: SourceId,
	content: SourceContent,
}

impl SourceFile {
	pub fn new(id: SourceId, path: impl Into<String>, content: impl Into<String>) -> Self {
		let path = path.into().into();
		let content = SourceContent::new(path, content.into().into_boxed_str());
		Self::from_raw_parts(id, content)
	}

	pub(super) const fn from_raw_parts(id: SourceId, content: SourceContent) -> Self {
		Self { id, content }
	}

	#[must_use]
	pub const fn id(&self) -> SourceId {
		self.id
	}

	#[must_use]
	pub const fn content(&self) -> &SourceContent {
		&self.content
	}

	#[must_use]
	pub fn line_count(&self) -> usize {
		self.last_line_index().to_usize() + 1
	}

	#[must_use]
	pub fn source_span(&self) -> SourceSpan {
		let range = self.source_range();
		SourceSpan::new(self.id, range.start.0..range.end.0)
	}

	#[must_use]
	pub fn line_column_to_span(&self, line: u32, column: u32) -> Option<SourceSpan> {
		let line_index = LineIndex::from(line.saturating_sub(1));
		let column_index = ColumnIndex::from(column.saturating_sub(1));
		let offset = self.line_column_to_offset(line_index, column_index)?;
		Some(SourceSpan::at(self.id, offset.0))
	}

	#[must_use]
	pub fn location(&self, span: SourceSpan) -> FileLineColumn {
		assert_eq!(span.source_id(), self.id(), "mismatched source ids");

		self.content()
			.location(ByteIndex(span.into_range().start))
			.expect("invalid source span: starting byte is out of bounds")
	}
}

impl AsRef<str> for SourceFile {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for SourceFile {
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

#[cfg(feature = "std")]
impl AsRef<std::path::Path> for SourceFile {
	fn as_ref(&self) -> &std::path::Path {
		self.path()
	}
}

impl Deref for SourceFile {
	type Target = SourceContent;

	fn deref(&self) -> &Self::Target {
		self.content()
	}
}

#[cfg(feature = "diagnostics")]
impl miette::SourceCode for SourceFile {
	fn read_span<'a>(
		&'a self,
		span: &miette::SourceSpan,
		context_lines_before: usize,
		context_lines_after: usize,
	) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
		let mut start =
			u32::try_from(span.offset()).map_err(|_| miette::MietteError::OutOfBounds)?;
		let len = u32::try_from(span.len()).map_err(|_| miette::MietteError::OutOfBounds)?;
		let mut end = start
			.checked_add(len)
			.ok_or(miette::MietteError::OutOfBounds)?;
		if context_lines_before > 0 {
			let line_index = self.content.line_index(start.into());
			let start_line_index = line_index.saturating_sub(context_lines_before as u32);
			start = self
				.content
				.line_start(start_line_index)
				.map_or(0, ByteIndex::to_u32);
		}
		if context_lines_after > 0 {
			let line_index = self.content.line_index(end.into());
			let end_line_index = line_index
				.checked_add(context_lines_after as u32)
				.ok_or(miette::MietteError::OutOfBounds)?;
			end = self.content.line_range(end_line_index).map_or_else(
				|| self.content.source_range().end.to_u32(),
				|range| range.end.to_u32(),
			);
		}

		Ok(Box::new(ScopedSourceFileRef {
			file: self,
			span: miette::SourceSpan::new((start as usize).into(), end.abs_diff(start) as usize),
		}))
	}
}

#[derive(Debug, Clone)]
pub struct SourceFileRef {
	file: Arc<SourceFile>,
	span: SourceSpan,
}

impl SourceFileRef {
	pub fn new(file: Arc<SourceFile>, span: impl Into<Range<u32>>) -> Self {
		let span = span.into();
		let end = core::cmp::min(span.end, file.len() as u32);
		let span = SourceSpan::new(file.id(), span.start..end);
		Self { file, span }
	}

	#[must_use]
	pub fn source_file(&self) -> Arc<SourceFile> {
		Arc::clone(&self.file)
	}

	#[cfg(feature = "std")]
	#[must_use]
	pub fn path(&self) -> &std::path::Path {
		self.file.path()
	}

	#[must_use]
	pub fn name(&self) -> &str {
		self.file.content.path.as_ref()
	}

	#[must_use]
	pub const fn span(&self) -> SourceSpan {
		self.span
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		self.file.source_slice(self.span).unwrap()
	}

	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		self.as_str().as_bytes()
	}

	#[must_use]
	pub const fn len(&self) -> usize {
		self.span.len()
	}

	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.span.is_empty()
	}
}

impl AsRef<str> for SourceFileRef {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for SourceFileRef {
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

#[cfg(feature = "diagnostics")]
impl From<&SourceFileRef> for miette::SourceSpan {
	fn from(value: &SourceFileRef) -> Self {
		value.span.into()
	}
}

impl Eq for SourceFileRef {}

impl Hash for SourceFileRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state);
	}
}

impl Ord for SourceFileRef {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.as_str().cmp(other.as_str())
	}
}

impl PartialEq for SourceFileRef {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

impl PartialOrd for SourceFileRef {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

#[cfg(feature = "diagnostics")]
impl miette::SourceCode for SourceFileRef {
	fn read_span<'a>(
		&'a self,
		span: &miette::SourceSpan,
		context_lines_before: usize,
		context_lines_after: usize,
	) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
		self.file
			.read_span(span, context_lines_before, context_lines_after)
	}
}

#[cfg(feature = "diagnostics")]
struct ScopedSourceFileRef<'a> {
	file: &'a SourceFile,
	span: miette::SourceSpan,
}

#[cfg(feature = "diagnostics")]
impl<'a> miette::SpanContents<'a> for ScopedSourceFileRef<'a> {
	fn data(&self) -> &'a [u8] {
		let start = self.span.offset();
		let end = start + self.span.len();
		&self.file.as_bytes()[start..end]
	}

	fn span(&self) -> &miette::SourceSpan {
		&self.span
	}

	fn line(&self) -> usize {
		let offset = self.span.offset() as u32;
		self.file.line_index(offset.into()).to_usize()
	}

	fn column(&self) -> usize {
		let start = self.span.offset() as u32;
		let end = start + self.span.len() as u32;
		let span = SourceSpan::new(self.file.id(), start..end);
		let loc = self.file.location(span);
		loc.column.saturating_sub(1) as usize
	}

	fn line_count(&self) -> usize {
		self.file.line_count()
	}

	fn name(&self) -> Option<&str> {
		Some(&self.file.content.path)
	}

	fn language(&self) -> Option<&str> {
		None
	}
}
