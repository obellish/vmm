#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod dsa;
pub mod hash;
pub mod merkle;
pub mod rand;
pub mod utils;

pub use winter_math::{
	FieldElement, StarkField,
	fields::{CubeExtension, QuadExtension, f64::BaseElement as Felt},
};

pub const WORD_SIZE: usize = 4;

pub const ZERO: Felt = Felt::ZERO;

pub const ONE: Felt = Felt::ONE;

pub const EMPTY_WORD: Word = [ZERO; WORD_SIZE];

pub type Word = [Felt; WORD_SIZE];
