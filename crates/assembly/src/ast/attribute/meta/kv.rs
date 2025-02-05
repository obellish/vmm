use alloc::collections::BTreeMap;
use core::{
	borrow::Borrow,
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
};

use super::MetaExpr;
use crate::{SourceSpan, Spanned, ast::Ident};

#[derive(Clone)]
pub struct MetaKeyValue {
	pub span: SourceSpan,
	pub name: Ident,
	pub items: BTreeMap<Ident, MetaExpr>,
}

impl MetaKeyValue {
	pub fn new<K, V>(name: Ident, items: impl IntoIterator<Item = (K, V)>) -> Self
	where
		K: Into<Ident>,
		V: Into<MetaExpr>,
	{
		let items = items
			.into_iter()
			.map(|(k, v)| (k.into(), v.into()))
			.collect();
		Self {
			span: SourceSpan::default(),
			name,
			items,
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

	pub fn contains_key<Q>(&self, key: &Q) -> bool
	where
		Ident: Borrow<Q> + Ord,
		Q: ?Sized + Ord,
	{
		self.items.contains_key(key)
	}

	pub fn get<Q>(&self, key: &Q) -> Option<&MetaExpr>
	where
		Ident: Borrow<Q> + Ord,
		Q: ?Sized + Ord,
	{
		self.items.get(key)
	}

	pub fn insert(&mut self, key: impl Into<Ident>, value: impl Into<MetaExpr>) {
		self.items.insert(key.into(), value.into());
	}

	pub fn remove<Q>(&mut self, key: &Q) -> Option<MetaExpr>
	where
		Ident: Borrow<Q> + Ord,
		Q: ?Sized + Ord,
	{
		self.items.remove(key)
	}

	pub fn entry(
		&mut self,
		key: Ident,
	) -> alloc::collections::btree_map::Entry<'_, Ident, MetaExpr> {
		self.items.entry(key)
	}

	pub fn iter(&self) -> impl Iterator<Item = (&Ident, &MetaExpr)> {
		self.items.iter()
	}
}

impl Debug for MetaKeyValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("MetaKeyValue")
			.field("name", &self.name)
			.field("items", &self.items)
			.finish_non_exhaustive()
	}
}

impl Eq for MetaKeyValue {}

impl Hash for MetaKeyValue {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.items.hash(state);
	}
}

impl IntoIterator for MetaKeyValue {
	type IntoIter = alloc::collections::btree_map::IntoIter<Ident, MetaExpr>;
	type Item = (Ident, MetaExpr);

	fn into_iter(self) -> Self::IntoIter {
		self.items.into_iter()
	}
}

impl Ord for MetaKeyValue {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.name
			.cmp(&other.name)
			.then_with(|| self.items.cmp(&other.items))
	}
}

impl PartialEq for MetaKeyValue {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.items == other.items
	}
}

impl PartialOrd for MetaKeyValue {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Spanned for MetaKeyValue {
	fn span(&self) -> SourceSpan {
		self.span
	}
}
