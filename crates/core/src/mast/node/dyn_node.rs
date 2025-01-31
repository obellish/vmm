use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
	Felt, OPCODE_DYN, OPCODE_DYNCALL,
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest},
	prettier::{Document, PrettyPrint, const_text, nl},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynNode {
	is_dyncall: bool,
	before_enter: Vec<DecoratorId>,
	after_exit: Vec<DecoratorId>,
}

impl DynNode {
	pub const DYNCALL_DOMAIN: Felt = Felt::new(OPCODE_DYNCALL as u64);
	pub const DYN_DOMAIN: Felt = Felt::new(OPCODE_DYN as u64);

	const fn new(is_dyncall: bool) -> Self {
		Self {
			is_dyncall,
			before_enter: Vec::new(),
			after_exit: Vec::new(),
		}
	}

	#[must_use]
	pub const fn r#dyn() -> Self {
		Self::new(false)
	}

	#[must_use]
	pub const fn dyncall() -> Self {
		Self::new(true)
	}

	#[must_use]
	pub const fn is_dyncall(&self) -> bool {
		self.is_dyncall
	}

	#[must_use]
	pub const fn domain(&self) -> Felt {
		if self.is_dyncall() {
			Self::DYNCALL_DOMAIN
		} else {
			Self::DYN_DOMAIN
		}
	}

	#[must_use]
	pub const fn digest(&self) -> RpoDigest {
		if self.is_dyncall() {
			RpoDigest::new([
				Felt::new(8_751_004_906_421_739_448),
				Felt::new(13_469_709_002_495_534_233),
				Felt::new(12_584_249_374_630_430_826),
				Felt::new(7_624_899_870_831_503_004),
			])
		} else {
			RpoDigest::new([
				Felt::new(8_115_106_948_140_260_551),
				Felt::new(13_491_227_816_952_616_836),
				Felt::new(15_015_806_788_322_198_710),
				Felt::new(16_575_543_461_540_527_115),
			])
		}
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
}
