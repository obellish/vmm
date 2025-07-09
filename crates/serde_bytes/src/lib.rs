#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod bytes;
mod de;
mod ser;

pub use self::{bytes::*, de::*, ser::*};
