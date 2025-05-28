#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "decode")]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "encode")]
pub mod encode;
mod imms;
mod regs;

pub use self::{imms::*, regs::*};
