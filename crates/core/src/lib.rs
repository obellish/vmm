#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod advice;
pub mod chiplets;
pub mod debuginfo;
pub mod errors;
mod kernel;
pub mod mast;
mod operations;
pub mod stack;
pub mod utils;

pub mod crypto {
	pub mod dsa {
		pub use miden_crypto::dsa::rpo_falcon512;
	}

	pub mod hash {
		pub use miden_crypto::hash::{
			Digest, ElementHasher, Hasher,
			blake::{Blake3_160, Blake3_192, Blake3_256, Blake3Digest},
			rpo::{Rpo256, RpoDigest},
			rpx::{Rpx256, RpxDigest},
		};
	}

	pub mod merkle {
		pub use miden_crypto::merkle::{
			DefaultMerkleStore, EmptySubtreeRoots, InnerNodeInfo, LeafIndex, MerkleError,
			MerklePath, MerkleStore, MerkleTree, Mmr, MmrPeaks, NodeIndex, PartialMerkleTree,
			RecordingMerkleStore, SMT_DEPTH, SimpleSmt, Smt, SmtProof, SmtProofError, StoreNode,
		};
	}

	pub mod random {
		pub use miden_crypto::rand::{
			RandomCoin, RandomCoinError, RpoRandomCoin, RpxRandomCoin, WinterRandomCoin,
		};
	}
}

pub mod prettier {
	pub use vmm_formatting::{hex, prettier::*, pretty_via_display, pretty_via_to_string};

	pub fn pretty_print_csv<'a, T>(items: impl IntoIterator<Item = &'a T>) -> Document
	where
		T: PrettyPrint + 'a,
	{
		let mut doc = Document::Empty;
		for (i, item) in items.into_iter().enumerate() {
			if i > 0 {
				doc += const_text(", ");
			}
			doc += item.render();
		}
		doc
	}
}

pub use miden_crypto::{
	EMPTY_WORD, Felt, FieldElement, ONE, QuadExtension, StarkField, WORD_SIZE, Word, ZERO,
};
pub use winter_math::{ExtensionOf, ToElements, polynom};

#[doc(inline)]
pub use self::{
	advice::AdviceMap,
	kernel::Kernel,
	operations::{
		AssemblyOp, DebugOptions, Decorator, DecoratorIterator, DecoratorList, DecoratorSlice,
		Operation, SignatureKind, opcode_constants::*,
	},
	stack::{StackInputs, StackOutputs},
};
