#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod de;
mod error;
pub mod format;
pub mod ser;
mod trace;
pub mod value;

#[doc(inline)]
pub use self::{
	error::*,
	format::{ContainerFormat, Format},
	trace::*,
	value::Value,
};
