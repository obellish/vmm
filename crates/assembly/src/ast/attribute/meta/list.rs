use alloc::vec::Vec;
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
};

use super::MetaExpr;
use crate::{SourceSpan, Spanned, ast::Ident};

#[derive(Clone)]
pub struct MetaList {
	pub span: SourceSpan,
	pub name: Ident,
	pub items: Vec<MetaExpr>,
}

impl MetaList {
	pub fn new<V>(name: Ident, items: impl IntoIterator<Item = V>) -> Self
	where
		V: Into<MetaExpr>,
	{
		Self {
			span: SourceSpan::default(),
			name,
			items: items.into_iter().map(Into::into).collect(),
		}
	}

	#[must_use]
	pub const fn with_span(mut self, span: SourceSpan) -> Self {
		self.span = span;
		self
	}

	#[must_use]
	pub fn name(&self) -> &str {
		self.id()
	}

	#[must_use]
	pub const fn id(&self) -> &Ident {
		&self.name
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.items.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	#[must_use]
	pub fn as_slice(&self) -> &[MetaExpr] {
		&self.items
	}

	pub fn as_mut_slice(&mut self) -> &mut [MetaExpr] {
		&mut self.items
	}
}

impl Debug for MetaList {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("MetaList")
			.field("name", &self.name)
			.field("items", &self.items)
			.finish_non_exhaustive()
	}
}

impl Eq for MetaList {}

impl Hash for MetaList {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.items.hash(state);
	}
}

impl Ord for MetaList {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.name
			.cmp(&other.name)
			.then_with(|| self.items.cmp(&other.items))
	}
}

impl PartialEq for MetaList {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.items == other.items
	}
}

impl PartialOrd for MetaList {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Spanned for MetaList {
	fn span(&self) -> SourceSpan {
		self.span
	}
}
