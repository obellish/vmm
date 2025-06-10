#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(
	feature = "nightly",
	feature(mixed_integer_ops_unsigned_sub, strict_overflow_ops)
)]
#![no_std]

mod checked;
mod identity;
pub mod ops;
mod sat;
#[cfg(feature = "nightly")]
mod strict;
mod unchecked;
mod wrap;

#[cfg(feature = "nightly")]
pub use self::strict::*;
pub use self::{checked::*, identity::*, sat::*, unchecked::*, wrap::*};
