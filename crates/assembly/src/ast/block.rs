use alloc::vec::Vec;
use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	slice,
};

use vmm_core::prettier::{Document, PrettyPrint, indent, nl};

use super::Op;
use crate::{SourceSpan, Span, Spanned, ast::Instruction};

#[derive(Default, Clone)]
pub struct Block {
	span: SourceSpan,
	body: Vec<Op>,
}

impl Block {
	#[must_use]
	pub const fn new(body: Vec<Op>, span: SourceSpan) -> Self {
		Self { span, body }
	}

	pub fn push(&mut self, op: Op) {
		self.body.push(op);
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.body.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.body.is_empty()
	}

	pub fn iter(&self) -> slice::Iter<'_, Op> {
		self.body.iter()
	}

	pub fn iter_mut(&mut self) -> slice::IterMut<'_, Op> {
		self.body.iter_mut()
	}
}

impl Debug for Block {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_list().entries(&self.body).finish()
	}
}

impl Display for Block {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl Eq for Block {}

impl<'a> IntoIterator for &'a Block {
	type IntoIter = slice::Iter<'a, Op>;
	type Item = &'a Op;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut Block {
	type IntoIter = slice::IterMut<'a, Op>;
	type Item = &'a mut Op;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl PartialEq for Block {
	fn eq(&self, other: &Self) -> bool {
		self.body == other.body
	}
}

impl PrettyPrint for Block {
	fn render(&self) -> Document {
		let default_body = [Op::Inst(Span::new(self.span, Instruction::Nop))];
		let body = match self.body.as_slice() {
			[] => default_body.as_slice().iter(),
			body => body.iter(),
		}
		.map(PrettyPrint::render)
		.reduce(|acc, doc| acc + nl() + doc);

		body.map(|body| indent(4, nl() + body)).unwrap_or_default()
	}
}

impl Spanned for Block {
	fn span(&self) -> SourceSpan {
		self.span
	}
}
