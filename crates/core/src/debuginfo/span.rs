use core::{
	borrow::Borrow,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::{Bound, Deref, DerefMut, Index, Range, RangeBounds},
};

use thiserror::Error;

use super::{ByteIndex, ByteOffset, SourceId};
use crate::{
	prettier::PrettyPrint,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Clone, Copy)]
pub struct Span<T> {
	span: SourceSpan,
	spanned: T,
}

impl<T> Span<T> {
	pub fn new(span: impl Into<SourceSpan>, spanned: T) -> Self {
		Self {
			span: span.into(),
			spanned,
		}
	}

	pub fn at(source_id: SourceId, offset: usize, spanned: T) -> Self {
		let offset = u32::try_from(offset).expect("invalid source offset: too large");
		Self {
			span: SourceSpan::at(source_id, offset),
			spanned,
		}
	}

	pub fn unknown(spanned: T) -> Self {
		Self {
			span: SourceSpan::default(),
			spanned,
		}
	}

	pub const fn span(&self) -> SourceSpan {
		self.span
	}

	pub const fn inner(&self) -> &T {
		&self.spanned
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.spanned
	}

	pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Span<U> {
		Span {
			span: self.span,
			spanned: f(self.spanned),
		}
	}

	pub fn as_deref<U: ?Sized>(&self) -> Span<&U>
	where
		T: Deref<Target = U>,
	{
		Span {
			span: self.span,
			spanned: &*self.spanned,
		}
	}

	pub const fn as_ref(&self) -> Span<&T> {
		Span {
			span: self.span,
			spanned: &self.spanned,
		}
	}

	pub fn shift(&mut self, count: ByteOffset) {
		self.span.start += count;
		self.extend(count);
	}

	pub fn extend(&mut self, count: ByteOffset) {
		self.span.end += count;
	}

	pub fn into_parts(self) -> (SourceSpan, T) {
		(self.span, self.spanned)
	}

	pub fn into_inner(self) -> T {
		self.spanned
	}
}

impl<T: Deserializable> Span<T> {
	pub fn read_from_with_options<R: ByteReader>(
		source: &mut R,
		debug: bool,
	) -> Result<Self, DeserializationError> {
		let span = if debug {
			SourceSpan::read_from(source)?
		} else {
			SourceSpan::default()
		};
		let spanned = T::read_from(source)?;
		Ok(Self { span, spanned })
	}
}

impl<T: Serializable> Span<T> {
	pub fn write_into_with_options<W: ByteWriter>(&self, target: &mut W, debug: bool) {
		if debug {
			self.span.write_into(target);
		}

		self.inner().write_into(target);
	}
}

impl<T: ?Sized, U> AsRef<T> for Span<U>
where
	U: AsRef<T>,
{
	fn as_ref(&self) -> &T {
		self.inner().as_ref()
	}
}

impl<T, S> Borrow<T> for Span<S>
where
	T: Borrow<str>,
	S: Borrow<T>,
{
	fn borrow(&self) -> &T {
		self.inner().borrow()
	}
}

impl<T: Debug> Debug for Span<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(self.inner(), f)
	}
}

impl<T> Deref for Span<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.inner()
	}
}

impl<T> DerefMut for Span<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner_mut()
	}
}

impl<T: Deserializable> Deserializable for Span<T> {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		Self::read_from_with_options(source, true)
	}
}

impl<T: Display> Display for Span<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(self.inner(), f)
	}
}

impl<T: Eq> Eq for Span<T> {}

impl<T: Hash> Hash for Span<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner().hash(state);
	}
}

impl<T: Ord> Ord for Span<T> {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.inner().cmp(other)
	}
}

impl<T: PartialEq> PartialEq for Span<T> {
	fn eq(&self, other: &Self) -> bool {
		self == other.inner()
	}
}

impl<T: PartialEq> PartialEq<T> for Span<T> {
	fn eq(&self, other: &T) -> bool {
		self.inner().eq(other)
	}
}

impl<T: PartialOrd> PartialOrd for Span<T> {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		self.inner().partial_cmp(other)
	}
}

impl<T: PrettyPrint> PrettyPrint for Span<T> {
	fn render(&self) -> crate::prettier::Document {
		self.inner().render()
	}
}

impl<T: Serializable> Serializable for Span<T> {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.write_into_with_options(target, true);
	}
}

impl<T> Spanned for Span<T> {
	fn span(&self) -> SourceSpan {
		self.span
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceSpan {
	source_id: SourceId,
	start: ByteIndex,
	end: ByteIndex,
}

impl SourceSpan {
	pub const UNKNOWN: Self = Self {
		source_id: SourceId::UNKNOWN,
		start: ByteIndex::new(0),
		end: ByteIndex::new(0),
	};

	pub fn new<B>(source_id: SourceId, range: Range<B>) -> Self
	where
		B: Into<ByteIndex>,
	{
		Self {
			source_id,
			start: range.start.into(),
			end: range.end.into(),
		}
	}

	pub fn at(source_id: SourceId, offset: impl Into<ByteIndex>) -> Self {
		let offset = offset.into();
		Self {
			source_id,
			start: offset,
			end: offset,
		}
	}

	pub fn try_from_range(
		source_id: SourceId,
		range: Range<usize>,
	) -> Result<Self, InvalidByteIndexRange> {
		const MAX: usize = u32::MAX as usize;
		if range.start > MAX || range.end > MAX {
			Err(InvalidByteIndexRange)
		} else {
			Ok(Self {
				source_id,
				start: ByteIndex::from(range.start as u32),
				end: ByteIndex::from(range.end as u32),
			})
		}
	}

	#[must_use]
	pub const fn is_unknown(self) -> bool {
		self.source_id.is_unknown()
	}

	#[must_use]
	pub const fn source_id(self) -> SourceId {
		self.source_id
	}

	#[must_use]
	pub const fn start(self) -> ByteIndex {
		self.start
	}

	#[must_use]
	pub const fn end(self) -> ByteIndex {
		self.end
	}

	#[must_use]
	pub const fn len(self) -> usize {
		self.end.to_usize() - self.start.to_usize()
	}

	#[must_use]
	pub const fn is_empty(self) -> bool {
		matches!(self.len(), 0)
	}

	#[must_use]
	pub const fn into_range(self) -> Range<u32> {
		self.start.to_u32()..self.end.to_u32()
	}

	#[must_use]
	pub const fn into_slice_index(self) -> Range<usize> {
		self.start.to_usize()..self.end.to_usize()
	}
}

impl Deserializable for SourceSpan {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let source_id = SourceId::new_unchecked(source.read_u32()?);
		let start = ByteIndex::from(source.read_u32()?);
		let end = ByteIndex::from(source.read_u32()?);

		Ok(Self {
			source_id,
			start,
			end,
		})
	}
}

impl From<SourceSpan> for Range<u32> {
	fn from(value: SourceSpan) -> Self {
		value.into_range()
	}
}

impl From<SourceSpan> for Range<usize> {
	fn from(value: SourceSpan) -> Self {
		value.into_slice_index()
	}
}

#[cfg(feature = "diagnostics")]
impl From<SourceSpan> for miette::SourceSpan {
	fn from(value: SourceSpan) -> Self {
		Self::new(
			miette::SourceOffset::from(value.start().to_usize()),
			value.len(),
		)
	}
}

impl Index<SourceSpan> for [u8] {
	type Output = Self;

	fn index(&self, index: SourceSpan) -> &Self::Output {
		&self[index.into_slice_index()]
	}
}

impl RangeBounds<ByteIndex> for SourceSpan {
	fn start_bound(&self) -> Bound<&ByteIndex> {
		Bound::Included(&self.start)
	}

	fn end_bound(&self) -> Bound<&ByteIndex> {
		Bound::Excluded(&self.end)
	}
}

impl Serializable for SourceSpan {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u32(self.source_id.to_u32());
		target.write_u32(self.start.into());
		target.write_u32(self.end.into());
	}
}

impl Spanned for SourceSpan {
	fn span(&self) -> SourceSpan {
		*self
	}
}

#[derive(Debug, Error)]
#[error("invalid byte index range: maximum supported byte index is 2^32")]
pub struct InvalidByteIndexRange;

pub trait Spanned {
	fn span(&self) -> SourceSpan;
}

impl<T> Spanned for alloc::boxed::Box<T>
where
	T: ?Sized + Spanned,
{
	fn span(&self) -> SourceSpan {
		(**self).span()
	}
}

impl<T> Spanned for alloc::rc::Rc<T>
where
	T: ?Sized + Spanned,
{
	fn span(&self) -> SourceSpan {
		(**self).span()
	}
}

impl<T> Spanned for alloc::sync::Arc<T>
where
	T: ?Sized + Spanned,
{
	fn span(&self) -> SourceSpan {
		(**self).span()
	}
}
