use alloc::{collections::BTreeSet, string::String};
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use vmm_core::prettier::{Document, PrettyPrint, const_text, display, indent, nl, text};

use super::ProcedureName;
use crate::{
	SourceSpan, Span, Spanned,
	ast::{Attribute, AttributeSet, Block, Invoke},
};

#[derive(Clone)]
pub struct Procedure {
	span: SourceSpan,
	docs: Option<Span<String>>,
	attrs: AttributeSet,
	name: ProcedureName,
	visibility: Visibility,
	num_locals: u16,
	body: Block,
	pub(super) invoked: BTreeSet<Invoke>,
}

impl Procedure {
	#[must_use]
	pub const fn new(
		span: SourceSpan,
		visibility: Visibility,
		name: ProcedureName,
		num_locals: u16,
		body: Block,
	) -> Self {
		Self {
			span,
			docs: None,
			attrs: AttributeSet::new(),
			name,
			visibility,
			num_locals,
			invoked: BTreeSet::new(),
			body,
		}
	}

	#[must_use]
	pub fn with_docs(mut self, docs: Option<Span<String>>) -> Self {
		self.docs = docs;
		self
	}

	#[must_use]
	pub fn with_attributes(mut self, attrs: impl IntoIterator<Item = Attribute>) -> Self {
		self.attrs.extend(attrs);
		self
	}

	pub(crate) fn set_visibility(&mut self, visibility: Visibility) {
		self.visibility = visibility;
	}

	#[must_use]
	pub const fn name(&self) -> &ProcedureName {
		&self.name
	}

	#[must_use]
	pub const fn visibility(&self) -> Visibility {
		self.visibility
	}

	#[must_use]
	pub const fn num_locals(&self) -> u16 {
		self.num_locals
	}

	#[must_use]
	pub fn is_entrypoint(&self) -> bool {
		self.name.is_main()
	}

	#[must_use]
	pub const fn docs(&self) -> Option<&Span<String>> {
		self.docs.as_ref()
	}

	#[must_use]
	pub const fn attributes(&self) -> &AttributeSet {
		&self.attrs
	}

	pub fn attributes_mut(&mut self) -> &mut AttributeSet {
		&mut self.attrs
	}

	pub fn has_attribute(&self, name: impl AsRef<str>) -> bool {
		self.attrs.has(name)
	}

	pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<&Attribute> {
		self.attrs.get(name)
	}

	#[must_use]
	pub const fn body(&self) -> &Block {
		&self.body
	}

	pub fn body_mut(&mut self) -> &mut Block {
		&mut self.body
	}

	pub fn invoked<'a, 'b: 'a>(&'b self) -> impl Iterator<Item = &'a Invoke> + 'a {
		if self.invoked.is_empty() {
			InvokedIter::Empty
		} else {
			InvokedIter::NonEmpty(self.invoked.iter())
		}
	}
}

impl Debug for Procedure {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Procedure")
			.field("docs", &self.docs)
			.field("attrs", &self.attrs)
			.field("name", &self.name)
			.field("visibility", &self.visibility)
			.field("num_locals", &self.num_locals)
			.field("body", &self.body)
			.field("invoked", &self.invoked)
			.finish_non_exhaustive()
	}
}

impl Eq for Procedure {}

impl Extend<Invoke> for Procedure {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = Invoke>,
	{
		self.invoked.extend(iter);
	}
}

impl PartialEq for Procedure {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.visibility == other.visibility
			&& self.num_locals == other.num_locals
			&& self.body == other.body
			&& self.attrs == other.attrs
			&& self.docs == other.docs
	}
}

impl PrettyPrint for Procedure {
	fn render(&self) -> Document {
		let mut doc = Document::Empty;
		if let Some(docs) = self.docs.as_deref() {
			doc = docs
				.lines()
				.map(text)
				.reduce(|acc, line| acc + nl() + const_text("#! ") + line)
				.unwrap_or_default();
		}

		if !self.attrs.is_empty() {
			doc = self
				.attrs
				.iter()
				.map(PrettyPrint::render)
				.reduce(|acc, attr| acc + nl() + attr)
				.unwrap_or_default();
		}

		doc += display(self.visibility) + const_text(".") + display(&self.name);
		if self.num_locals > 0 {
			doc += const_text(".") + display(self.num_locals);
		}

		doc += indent(4, nl() + self.body.render()) + nl();

		doc + const_text("end") + nl() + nl()
	}
}

impl Spanned for Procedure {
	fn span(&self) -> SourceSpan {
		self.span
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Visibility {
	Public = 0,
	SysCall = 1,
	#[default]
	Private = 2,
}

impl Visibility {
	#[must_use]
	pub const fn is_exported(self) -> bool {
		matches!(self, Self::Public | Self::SysCall)
	}

	#[must_use]
	pub const fn is_syscall(self) -> bool {
		matches!(self, Self::SysCall)
	}
}

impl Display for Visibility {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(if self.is_exported() { "export" } else { "proc" })
	}
}

pub(crate) enum InvokedIter<'a, I>
where
	I: Iterator<Item = &'a Invoke> + 'a,
{
	Empty,
	NonEmpty(I),
}

impl<'a, I> Iterator for InvokedIter<'a, I>
where
	I: Iterator<Item = &'a Invoke> + 'a,
{
	type Item = <I as Iterator>::Item;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Empty => None,
			Self::NonEmpty(iter) => {
				let result = iter.next();
				if result.is_none() {
					*self = Self::Empty;
				}
				result
			}
		}
	}
}
