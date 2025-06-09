#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(mixed_integer_ops_unsigned_sub))]
#![no_std]

pub mod ops;
mod sat;
mod wrap;

pub use self::{sat::*, wrap::*};
