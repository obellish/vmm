use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
	Felt, OPCODE_LOOP,
	chiplets::hasher,
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest, MastForestError, MastNodeId},
	prettier::{Document, PrettyPrint, const_text, indent, nl},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopNode {
	body: MastNodeId,
	digest: RpoDigest,
	before_enter: Vec<DecoratorId>,
	after_exit: Vec<DecoratorId>,
}

impl LoopNode {
	pub const DOMAIN: Felt = Felt::new(OPCODE_LOOP as u64);

	pub fn new(body: MastNodeId, mast_forest: &MastForest) -> Result<Self, MastForestError> {
		if body.as_usize() >= mast_forest.nodes.len() {
			return Err(MastForestError::NodeIdOverflow(
				body,
				mast_forest.nodes.len(),
			));
		}

		let digest = {
			let body_hash = mast_forest[body].digest();

			hasher::merge_in_domain(&[body_hash, RpoDigest::default()], Self::DOMAIN)
		};

		Ok(Self::new_unchecked(body, digest))
	}

	#[must_use]
	pub const fn new_unchecked(body: MastNodeId, digest: RpoDigest) -> Self {
		Self {
			body,
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
	pub const fn body(&self) -> MastNodeId {
		self.body
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
		LoopNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}

	pub(super) fn to_pretty_print<'a>(
		&'a self,
		mast_forest: &'a MastForest,
	) -> impl PrettyPrint + 'a {
		LoopNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}
}

struct LoopNodePrettyPrint<'a> {
	node: &'a LoopNode,
	mast_forest: &'a MastForest,
}

impl Display for LoopNodePrettyPrint<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for LoopNodePrettyPrint<'_> {
	fn render(&self) -> Document {
		let pre_decorators = {
			let mut pre_decorators = self
				.node
				.before_enter()
				.iter()
				.map(|&decorator_id| self.mast_forest[decorator_id].render())
				.reduce(|acc, doc| acc + const_text(" ") + doc)
				.unwrap_or_default();

			if !pre_decorators.is_empty() {
				pre_decorators += nl();
			}

			pre_decorators
		};

		let post_decorators = {
			let mut post_decorators = self
				.node
				.after_exit()
				.iter()
				.map(|&decorator_id| self.mast_forest[decorator_id].render())
				.reduce(|acc, doc| acc + const_text(" ") + doc)
				.unwrap_or_default();

			if !post_decorators.is_empty() {
				post_decorators += nl();
			}

			post_decorators
		};

		let loop_body = self.mast_forest[self.node.body()].to_pretty_print(self.mast_forest);

		pre_decorators
			+ indent(4, const_text("while.true") + nl() + loop_body.render())
			+ nl() + const_text("end")
			+ post_decorators
	}
}
