#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod global;
mod hash;
mod ref_count;
mod tree;

pub use self::{
	hash::{HashInterner, InternedHash},
	tree::{InternedOrd, OrdInterner},
};
