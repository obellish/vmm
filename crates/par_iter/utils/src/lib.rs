#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod backoff;
mod cache_padded;
#[cfg(feature = "std")]
pub mod sync;

pub use self::{backoff::Backoff, cache_padded::CachePadded};
