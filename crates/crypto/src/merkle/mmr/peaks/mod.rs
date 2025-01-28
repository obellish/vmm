#[cfg(feature = "serde")]
mod serde;

use alloc::vec::Vec;

use super::{MmrError, MmrProof, PartialMmr};
use crate::{
	Felt, Word, ZERO,
	hash::rpo::{Rpo256, RpoDigest},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmrPeaks {
	num_leaves: usize,
	peaks: Vec<RpoDigest>,
}

impl MmrPeaks {
	pub fn new(num_leaves: usize, peaks: Vec<RpoDigest>) -> Result<Self, MmrError> {
		if num_leaves.count_ones() as usize != peaks.len() {
			return Err(MmrError::InvalidPeaks(format!(
				"number of one bits in leaves is {} which does not equal peak length {}",
				num_leaves.count_ones(),
				peaks.len()
			)));
		}

		Ok(Self { num_leaves, peaks })
	}

	#[must_use]
	pub const fn num_leaves(&self) -> usize {
		self.num_leaves
	}

	#[must_use]
	pub fn num_peaks(&self) -> usize {
		self.peaks.len()
	}

	#[must_use]
	pub fn peaks(&self) -> &[RpoDigest] {
		&self.peaks
	}

	pub fn get_peak(&self, peak_idx: usize) -> Result<&RpoDigest, MmrError> {
		self.peaks
			.get(peak_idx)
			.ok_or_else(|| MmrError::PeakOutOfBounds {
				peak_idx,
				peaks_len: self.num_peaks(),
			})
	}

	#[must_use]
	pub fn into_parts(self) -> (usize, Vec<RpoDigest>) {
		(self.num_leaves, self.peaks)
	}

	#[must_use]
	pub fn hash_peaks(&self) -> RpoDigest {
		Rpo256::hash_elements(&self.flatten_and_pad_peaks())
	}

	pub fn verify(&self, value: RpoDigest, opening: MmrProof) -> Result<(), MmrError> {
		let root = self.get_peak(opening.peak_index())?;
		opening
			.merkle_path
			.verify(opening.relative_pos() as u64, value, *root)
			.map_err(MmrError::InvalidMerklePath)
	}

	pub fn flatten_and_pad_peaks(&self) -> Vec<Felt> {
		let num_peaks = self.num_peaks();

		let len = if num_peaks < 16 {
			64
		} else if matches!(num_peaks % 2, 1) {
			(num_peaks + 1) * 4
		} else {
			num_peaks * 4
		};

		let mut elements = Vec::with_capacity(len);
		elements.extend_from_slice(
			&self
				.peaks
				.as_slice()
				.iter()
				.map(Into::into)
				.collect::<Vec<Word>>()
				.concat(),
		);
		elements.resize(len, ZERO);
		elements
	}
}

impl IntoIterator for MmrPeaks {
	type IntoIter = alloc::vec::IntoIter<RpoDigest>;
	type Item = RpoDigest;

	fn into_iter(self) -> Self::IntoIter {
		self.peaks.into_iter()
	}
}

impl TryFrom<PartialMmr> for MmrPeaks {
	type Error = MmrError;

	fn try_from(value: PartialMmr) -> Result<Self, Self::Error> {
		Self::new(value.forest, value.peaks)
	}
}
