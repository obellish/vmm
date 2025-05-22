#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod insert_or_push;
mod is_closed_range;

pub use self::{insert_or_push::InsertOrPush, is_closed_range::IsClosedRange};
