#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod char;
mod error;
mod iter;
mod owned;
mod pattern;
mod slice;
mod validations;

pub use self::{char::*, error::*, iter::*, owned::*, pattern::*, slice::*};
