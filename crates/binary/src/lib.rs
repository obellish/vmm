#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod de;
pub mod ser;
mod tag;
mod value;

pub use self::{tag::*, value::*};
