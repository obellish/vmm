#![allow(clippy::return_self_not_must_use)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(never_type))]
#![no_std]

mod conv;
mod pipe;
mod tap;

pub use self::{conv::*, pipe::*, tap::*};

#[cfg(test)]
mod tests {}
