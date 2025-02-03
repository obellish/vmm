use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest},
	prettier::{Document, PrettyPrint, const_text, hex::ToHex, nl, text},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalNode {
	digest: RpoDigest,
	before_enter: Vec<DecoratorId>,
	after_exit: Vec<DecoratorId>,
}

impl ExternalNode {
	#[must_use]
	pub const fn new(digest: RpoDigest) -> Self {
		Self {
			digest,
			before_enter: Vec::new(),
			after_exit: Vec::new(),
		}
	}

	#[must_use]
	pub const fn digest(&self) -> RpoDigest {
		self.digest
	}

	#[must_use]
	pub fn before_enter(&self) -> &[DecoratorId] {
		&self.before_enter
	}

	#[must_use]
	pub fn after_exit(&self) -> &[DecoratorId] {
		&self.after_exit
	}

	pub fn set_before_enter(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		self.before_enter = decorator_ids.into_iter().collect();
	}

	pub fn set_after_exit(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		self.after_exit = decorator_ids.into_iter().collect();
	}

	pub(super) fn to_display<'a>(&'a self, mast_forest: &'a MastForest) -> impl Display + 'a {
		ExternalNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}

	pub(super) fn to_pretty_print<'a>(
		&'a self,
		mast_forest: &'a MastForest,
	) -> impl PrettyPrint + 'a {
		ExternalNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}
}

struct ExternalNodePrettyPrint<'a> {
	node: &'a ExternalNode,
	mast_forest: &'a MastForest,
}

impl ExternalNodePrettyPrint<'_> {
	fn concatenate_decorators(
		&self,
		decorator_ids: &[DecoratorId],
		prepend: Document,
		append: Document,
	) -> Document {
		let decorators = decorator_ids
			.iter()
			.map(|&decorator_id| self.mast_forest[decorator_id].render())
			.reduce(|acc, doc| acc + const_text(" ") + doc)
			.unwrap_or_default();

		if decorators.is_empty() {
			decorators
		} else {
			prepend + decorators + append
		}
	}

	fn single_line_pre_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.before_enter(), Document::Empty, const_text(" "))
	}

	fn single_line_post_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.after_exit(), const_text(" "), Document::Empty)
	}

	fn multi_line_pre_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.before_enter(), Document::Empty, nl())
	}

	fn multi_line_post_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.after_exit(), nl(), Document::Empty)
	}
}

impl Display for ExternalNodePrettyPrint<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for ExternalNodePrettyPrint<'_> {
	fn render(&self) -> Document {
		let external = const_text("external")
			+ const_text(".")
			+ text(self.node.digest().as_bytes().to_hex_with_prefix());

		let single_line = self.single_line_pre_decorators()
			+ external.clone()
			+ self.single_line_post_decorators();

		let multi_line =
			self.multi_line_pre_decorators() + external + self.multi_line_post_decorators();

		single_line | multi_line
	}
}
