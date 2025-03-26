#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod cesu8;
mod char;
mod error;
mod iter;
mod owned;
mod pattern;
#[cfg(feature = "serde")]
mod serde;
mod slice;
mod validations;

pub use self::{char::*, error::*, iter::*, owned::*, pattern::*, slice::*};
