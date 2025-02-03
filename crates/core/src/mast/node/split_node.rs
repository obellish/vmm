use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
	Felt, OPCODE_SPLIT,
	chiplets::hasher,
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest, MastForestError, MastNodeId},
	prettier::{Document, PrettyPrint, const_text, indent, nl},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitNode {
	branches: [MastNodeId; 2],
	digest: RpoDigest,
	before_enter: Vec<DecoratorId>,
	after_exit: Vec<DecoratorId>,
}

impl SplitNode {
	pub const DOMAIN: Felt = Felt::new(OPCODE_SPLIT as u64);

	pub fn new(
		branches: [MastNodeId; 2],
		mast_forest: &MastForest,
	) -> Result<Self, MastForestError> {
		let forest_len = mast_forest.nodes.len();
		if branches[0].as_usize() >= forest_len {
			return Err(MastForestError::NodeIdOverflow(branches[0], forest_len));
		} else if branches[1].as_usize() >= forest_len {
			return Err(MastForestError::NodeIdOverflow(branches[1], forest_len));
		}

		let digest = {
			let if_branch_hash = mast_forest[branches[0]].digest();
			let else_branch_hash = mast_forest[branches[1]].digest();

			hasher::merge_in_domain(&[if_branch_hash, else_branch_hash], Self::DOMAIN)
		};

		Ok(Self::new_unchecked(branches, digest))
	}

	#[must_use]
	pub const fn new_unchecked(branches: [MastNodeId; 2], digest: RpoDigest) -> Self {
		Self {
			branches,
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
	pub const fn on_true(&self) -> MastNodeId {
		self.branches[0]
	}

	#[must_use]
	pub const fn on_false(&self) -> MastNodeId {
		self.branches[1]
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
		SplitNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}

	pub(super) fn to_pretty_print<'a>(
		&'a self,
		mast_forest: &'a MastForest,
	) -> impl PrettyPrint + 'a {
		SplitNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}
}

struct SplitNodePrettyPrint<'a> {
	node: &'a SplitNode,
	mast_forest: &'a MastForest,
}

impl Display for SplitNodePrettyPrint<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for SplitNodePrettyPrint<'_> {
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

		let true_branch = self.mast_forest[self.node.on_true()].to_pretty_print(self.mast_forest);
		let false_branch = self.mast_forest[self.node.on_false()].to_pretty_print(self.mast_forest);

		let mut doc = pre_decorators;
		doc += indent(4, const_text("if.true") + nl() + true_branch.render()) + nl();
		doc += indent(4, const_text("else") + nl() + false_branch.render());
		doc += nl() + const_text("end");

		doc + post_decorators
	}
}
