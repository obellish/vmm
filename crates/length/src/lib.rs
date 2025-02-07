#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

#[macro_use]
extern crate alloc;

mod inner;
mod macros;

pub use self::inner::{Length, SmallLength};
