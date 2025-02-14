#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod errno;
#[cfg(feature = "std")]
pub mod tempfile;
