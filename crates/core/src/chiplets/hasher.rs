#![allow(clippy::trivially_copy_pass_by_ref)]

use crate::Felt;
pub use crate::crypto::hash::{Rpo256 as Hasher, RpoDigest as Digest};

pub const STATE_WIDTH: usize = Hasher::STATE_WIDTH;
pub const RATE_LEN: usize = 8;

#[inline]
#[must_use]
pub fn merge(values: &[Digest; 2]) -> Digest {
	Hasher::merge(values)
}

#[inline]
#[must_use]
pub fn merge_in_domain(values: &[Digest; 2], domain: Felt) -> Digest {
	Hasher::merge_in_domain(values, domain)
}

#[inline]
#[must_use]
pub fn hash_elements(elements: &[Felt]) -> Digest {
	Hasher::hash_elements(elements)
}

#[inline]
pub fn apply_round(state: &mut [Felt; STATE_WIDTH], round: usize) {
	Hasher::apply_round(state, round);
}

#[inline]
pub fn apply_permutation(state: &mut [Felt; STATE_WIDTH]) {
	Hasher::apply_permutation(state);
}
