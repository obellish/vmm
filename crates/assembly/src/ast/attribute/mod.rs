mod meta;
mod set;

use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use vmm_core::prettier::{Document, PrettyPrint, const_text, indent, nl, text};

pub use self::{
	meta::{Meta, MetaExpr, MetaItem, MetaKeyValue, MetaList, MetaRef},
	set::{AttributeSet, AttributeSetEntry, AttributeSetOccupiedEntry, AttributeSetVacantEntry},
};
use crate::{SourceSpan, Spanned, ast::Ident};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Attribute {
	Marker(Ident),
	List(MetaList),
	KeyValue(MetaKeyValue),
}

impl Attribute {
	pub fn new(name: Ident, metadata: impl Into<Meta>) -> Self {
		let metadata = metadata.into();
		match metadata {
			Meta::Unit => Self::Marker(name),
			Meta::List(items) => Self::List(MetaList {
				span: SourceSpan::default(),
				name,
				items,
			}),
			Meta::KeyValue(items) => Self::KeyValue(MetaKeyValue {
				span: SourceSpan::default(),
				name,
				items,
			}),
		}
	}

	pub fn from_iter<V>(name: Ident, metadata: impl IntoIterator<Item = V>) -> Self
	where
		Meta: FromIterator<V>,
	{
		Self::new(name, Meta::from_iter(metadata))
	}

	#[must_use]
	pub fn with_span(self, span: SourceSpan) -> Self {
		match self {
			Self::Marker(id) => Self::Marker(id.with_span(span)),
			Self::List(list) => Self::List(list.with_span(span)),
			Self::KeyValue(kv) => Self::KeyValue(kv.with_span(span)),
		}
	}

	#[must_use]
	pub fn name(&self) -> &str {
		self.id()
	}

	#[must_use]
	pub const fn id(&self) -> &Ident {
		match self {
			Self::Marker(id) => id,
			Self::List(list) => list.id(),
			Self::KeyValue(kv) => kv.id(),
		}
	}

	#[must_use]
	pub const fn is_marker(&self) -> bool {
		matches!(self, Self::Marker(_))
	}

	#[must_use]
	pub const fn is_list(&self) -> bool {
		matches!(self, Self::List(_))
	}

	#[must_use]
	pub const fn is_key_value(&self) -> bool {
		matches!(self, Self::KeyValue(_))
	}

	#[must_use]
	pub fn metadata(&self) -> Option<MetaRef<'_>> {
		match self {
			Self::Marker(_) => None,
			Self::List(list) => Some(MetaRef::List(&list.items)),
			Self::KeyValue(kv) => Some(MetaRef::KeyValue(&kv.items)),
		}
	}
}

impl Debug for Attribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Marker(id) => f.debug_tuple("Marker").field(&id).finish(),
			Self::List(meta) => Debug::fmt(&meta, f),
			Self::KeyValue(meta) => Debug::fmt(&meta, f),
		}
	}
}

impl Display for Attribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl From<Ident> for Attribute {
	fn from(value: Ident) -> Self {
		Self::Marker(value)
	}
}

impl<K, V> From<(K, V)> for Attribute
where
	K: Into<Ident>,
	V: Into<MetaExpr>,
{
	fn from(value: (K, V)) -> Self {
		let (key, value) = value;
		MetaList {
			name: key.into(),
			items: vec![value.into()],
			span: SourceSpan::default(),
		}
		.into()
	}
}

impl From<MetaList> for Attribute {
	fn from(value: MetaList) -> Self {
		Self::List(value)
	}
}

impl From<MetaKeyValue> for Attribute {
	fn from(value: MetaKeyValue) -> Self {
		Self::KeyValue(value)
	}
}

impl PrettyPrint for Attribute {
	fn render(&self) -> Document {
		let doc = text(format!("@{}", &self.name()));
		match self {
			Self::Marker(_) => doc,
			Self::List(meta) => {
				let singleline_items = meta
					.items
					.iter()
					.map(PrettyPrint::render)
					.reduce(|acc, item| acc + const_text(", ") + item)
					.unwrap_or_default();
				let multiline_items = indent(
					4,
					nl() + meta
						.items
						.iter()
						.map(PrettyPrint::render)
						.reduce(|acc, item| acc + nl() + item)
						.unwrap_or_default(),
				) + nl();
				doc + const_text("(") + (singleline_items | multiline_items) + const_text(")")
			}
			Self::KeyValue(meta) => {
				let singleline_items = meta
					.items
					.iter()
					.map(|(k, v)| text(k) + const_text(" = ") + v.render())
					.reduce(|acc, item| acc + const_text(", ") + item)
					.unwrap_or_default();
				let multiline_items = indent(
					4,
					nl() + meta
						.items
						.iter()
						.map(|(k, v)| text(k) + const_text(" = ") + v.render())
						.reduce(|acc, item| acc + nl() + item)
						.unwrap_or_default(),
				) + nl();

				doc + const_text("(") + (singleline_items | multiline_items) + const_text(")")
			}
		}
	}
}

impl Spanned for Attribute {
	fn span(&self) -> SourceSpan {
		match self {
			Self::Marker(id) => id.span(),
			Self::List(list) => list.span(),
			Self::KeyValue(kv) => kv.span(),
		}
	}
}
