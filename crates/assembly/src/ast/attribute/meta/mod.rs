mod expr;
mod kv;
mod list;

use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use vmm_core::Felt;

pub use self::{expr::MetaExpr, kv::MetaKeyValue, list::MetaList};
use crate::{SourceSpan, Span, ast::Ident, parser::HexEncodedValue};

#[derive(Clone, PartialEq, Eq)]
pub enum Meta {
	Unit,
	List(Vec<MetaExpr>),
	KeyValue(BTreeMap<Ident, MetaExpr>),
}

impl Meta {
	#[must_use]
	pub fn borrow(&self) -> Option<MetaRef<'_>> {
		match self {
			Self::Unit => None,
			Self::List(list) => Some(MetaRef::List(list)),
			Self::KeyValue(kv) => Some(MetaRef::KeyValue(kv)),
		}
	}
}

impl<I, V> From<I> for Meta
where
	Self: FromIterator<V>,
	I: IntoIterator<Item = V>,
{
	fn from(value: I) -> Self {
		value.into_iter().collect()
	}
}

impl FromIterator<MetaItem> for Meta {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = MetaItem>,
	{
		let mut iter = iter.into_iter();
		match iter.next() {
			None => Self::Unit,
			Some(MetaItem::Expr(expr)) => Self::List(
				core::iter::once(expr)
					.chain(iter.map(|item| match item {
						MetaItem::Expr(expr) => expr,
						MetaItem::KeyValue(..) => unsafe { core::hint::unreachable_unchecked() },
					}))
					.collect(),
			),
			Some(MetaItem::KeyValue(k, v)) => Self::KeyValue(
				core::iter::once((k, v))
					.chain(iter.map(|item| match item {
						MetaItem::KeyValue(k, v) => (k, v),
						MetaItem::Expr(..) => unsafe { core::hint::unreachable_unchecked() },
					}))
					.collect(),
			),
		}
	}
}

impl FromIterator<MetaExpr> for Meta {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = MetaExpr>,
	{
		Self::List(iter.into_iter().collect())
	}
}

impl FromIterator<(Ident, MetaExpr)> for Meta {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = (Ident, MetaExpr)>,
	{
		Self::KeyValue(iter.into_iter().collect())
	}
}

impl<'a> FromIterator<(&'a str, MetaExpr)> for Meta {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = (&'a str, MetaExpr)>,
	{
		iter.into_iter()
			.map(|(k, v)| {
				let k = Ident::new_unchecked(Span::new(SourceSpan::UNKNOWN, k.into()));
				(k, v)
			})
			.collect()
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MetaRef<'a> {
	List(&'a [MetaExpr]),
	KeyValue(&'a BTreeMap<Ident, MetaExpr>),
}

impl Debug for MetaRef<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::List(items) => write!(f, "{items:#?}"),
			Self::KeyValue(items) => write!(f, "{items:#?}"),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum MetaItem {
	Expr(MetaExpr),
	KeyValue(Ident, MetaExpr),
}

impl MetaItem {
	#[must_use]
	#[track_caller]
	pub fn unwrap_expr(self) -> MetaExpr {
		match self {
			Self::Expr(expr) => expr,
			Self::KeyValue(..) => unreachable!("tried to unwrap key-value as expression"),
		}
	}

	#[must_use]
	#[track_caller]
	pub fn unwrap_key_value(self) -> (Ident, MetaExpr) {
		match self {
			Self::KeyValue(k, v) => (k, v),
			Self::Expr(..) => unreachable!("tried to unwrap expression as key-value"),
		}
	}
}

impl<V> From<V> for MetaItem
where
	V: Into<MetaExpr>,
{
	fn from(value: V) -> Self {
		Self::Expr(value.into())
	}
}

impl<V> From<(Ident, V)> for MetaItem
where
	V: Into<MetaExpr>,
{
	fn from(value: (Ident, V)) -> Self {
		let (key, value) = value;
		Self::KeyValue(key, value.into())
	}
}

impl<V> From<(&str, V)> for MetaItem
where
	V: Into<MetaExpr>,
{
	fn from(value: (&str, V)) -> Self {
		let (key, value) = value;
		let key = Ident::new_unchecked(Span::new(SourceSpan::UNKNOWN, Arc::from(key)));
		Self::KeyValue(key, value.into())
	}
}

impl<V> From<(String, V)> for MetaItem
where
	V: Into<MetaExpr>,
{
	fn from(value: (String, V)) -> Self {
		let (key, value) = value;
		let key = Ident::new_unchecked(Span::new(SourceSpan::UNKNOWN, key.into()));
		Self::KeyValue(key, value.into())
	}
}
