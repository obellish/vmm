#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod errors;
