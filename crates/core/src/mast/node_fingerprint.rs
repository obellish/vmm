use alloc::{collections::BTreeMap, vec::Vec};

use super::{DecoratorId, MastForest, MastForestError, MastNode, MastNodeId};
use crate::{
	Operation,
	crypto::hash::{Blake3_256, Blake3Digest, Digest, RpoDigest},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MastNodeFingerprint {
	mast_root: RpoDigest,
	decorator_root: Option<DecoratorFingerprint>,
}

impl MastNodeFingerprint {
	#[must_use]
	pub const fn new(mast_root: RpoDigest) -> Self {
		Self::new_inner(mast_root, None)
	}

	#[must_use]
	pub const fn with_decorator_root(
		mast_root: RpoDigest,
		decorator_root: DecoratorFingerprint,
	) -> Self {
		Self::new_inner(mast_root, Some(decorator_root))
	}

	const fn new_inner(mast_root: RpoDigest, decorator_root: Option<DecoratorFingerprint>) -> Self {
		Self {
			mast_root,
			decorator_root,
		}
	}

	#[must_use]
	pub const fn mast_root(&self) -> &RpoDigest {
		&self.mast_root
	}

	pub fn from_mast_node(
		forest: &MastForest,
		hash_by_node_id: &BTreeMap<MastNodeId, Self>,
		node: &MastNode,
	) -> Result<Self, MastForestError> {
		match node {
			MastNode::Block(node) => {
				let mut bytes_to_hash = Vec::new();

				for &(idx, decorator_id) in node.decorators() {
					bytes_to_hash.extend(idx.to_le_bytes());
					bytes_to_hash.extend(forest[decorator_id].fingerprint().as_bytes());
				}

				for (op_idx, op) in node.operations().enumerate() {
					if let Operation::U32assert2(inner_value)
					| Operation::Assert(inner_value)
					| Operation::MpVerify(inner_value) = op
					{
						let op_idx: u32 = op_idx
							.try_into()
							.expect("there are more than 2^{32}-1 operations in basic block");

						bytes_to_hash.push(op.op_code());
						bytes_to_hash.extend(op_idx.to_le_bytes());
						bytes_to_hash.extend(inner_value.to_le_bytes());
					}
				}

				if bytes_to_hash.is_empty() {
					Ok(Self::new(node.digest()))
				} else {
					let decorator_root = Blake3_256::hash(&bytes_to_hash);
					Ok(Self::with_decorator_root(node.digest(), decorator_root))
				}
			}
			MastNode::Join(node) => fingerprint_from_parts(
				forest,
				hash_by_node_id,
				node.before_enter(),
				node.after_exit(),
				&[node.first(), node.second()],
				node.digest(),
			),
			MastNode::Split(node) => fingerprint_from_parts(
				forest,
				hash_by_node_id,
				node.before_enter(),
				node.after_exit(),
				&[node.on_true(), node.on_false()],
				node.digest(),
			),
			MastNode::Loop(node) => fingerprint_from_parts(
				forest,
				hash_by_node_id,
				node.before_enter(),
				node.after_exit(),
				&[node.body()],
				node.digest(),
			),
			MastNode::Call(node) => fingerprint_from_parts(
				forest,
				hash_by_node_id,
				node.before_enter(),
				node.after_exit(),
				&[node.callee()],
				node.digest(),
			),
			MastNode::Dyn(node) => fingerprint_from_parts(
				forest,
				hash_by_node_id,
				node.before_enter(),
				node.after_exit(),
				&[],
				node.digest(),
			),
			MastNode::External(node) => fingerprint_from_parts(
				forest,
				hash_by_node_id,
				node.before_enter(),
				node.after_exit(),
				&[],
				node.digest(),
			),
		}
	}
}

pub type DecoratorFingerprint = Blake3Digest<32>;

fn fingerprint_from_parts(
	forest: &MastForest,
	hash_by_node_id: &BTreeMap<MastNodeId, MastNodeFingerprint>,
	before_enter_ids: &[DecoratorId],
	after_exit_ids: &[DecoratorId],
	children_ids: &[MastNodeId],
	node_digest: RpoDigest,
) -> Result<MastNodeFingerprint, MastForestError> {
	let pre_decorator_hash_bytes = before_enter_ids
		.iter()
		.flat_map(|&id| forest[id].fingerprint().as_bytes());
	let post_decorator_hash_bytes = after_exit_ids
		.iter()
		.flat_map(|&id| forest[id].fingerprint().as_bytes());

	let children_decorator_roots = children_ids
		.iter()
		.filter_map(|child_id| {
			hash_by_node_id
				.get(child_id)
				.ok_or(MastForestError::ChildFingerprintMissing(*child_id))
				.map(|child_fingerprint| child_fingerprint.decorator_root)
				.transpose()
		})
		.collect::<Result<Vec<_>, _>>()?;

	if pre_decorator_hash_bytes.clone().next().is_none()
		&& post_decorator_hash_bytes.clone().next().is_none()
		&& children_decorator_roots.is_empty()
	{
		Ok(MastNodeFingerprint::new(node_digest))
	} else {
		let decorator_bytes_to_hash = pre_decorator_hash_bytes
			.chain(post_decorator_hash_bytes)
			.chain(
				children_decorator_roots
					.into_iter()
					.flat_map(|decorator_root| decorator_root.as_bytes()),
			)
			.collect::<Vec<_>>();

		let decorator_root = Blake3_256::hash(&decorator_bytes_to_hash);
		Ok(MastNodeFingerprint::with_decorator_root(
			node_digest,
			decorator_root,
		))
	}
}
