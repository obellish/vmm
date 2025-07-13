#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod ast;
mod common;
pub mod semantic;
pub mod types;

pub use self::common::*;
