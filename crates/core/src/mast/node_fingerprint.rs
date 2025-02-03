use alloc::{collections::BTreeMap, vec::Vec};

use super::{DecoratorId, MastForest, MastForestError, MastNode, MastNodeId};
use crate::crypto::hash::{Blake3_256, Blake3Digest, Digest, RpoDigest};

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
}

pub type DecoratorFingerprint = Blake3Digest<32>;
