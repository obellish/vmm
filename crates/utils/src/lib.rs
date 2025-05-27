#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod get_or_zero;
mod insert_or_push;

pub use self::{get_or_zero::GetOrZero, insert_or_push::InsertOrPush};
