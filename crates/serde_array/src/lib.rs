#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

mod array;
mod const_generics;

pub use self::{array::Array, const_generics::BigArray};
