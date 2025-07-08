#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod buffer;
mod config;
pub mod de;
mod error;
mod format;
mod io;
pub mod ser;

pub use self::{buffer::*, config::*, error::*, format::*, io::*};
