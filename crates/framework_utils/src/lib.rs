#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod diagnostics;
pub mod expect;
pub mod future;
pub mod hierarchy;
pub mod system;

pub mod prelude {
	pub use super::{expect::Expect, system::*};
}

#[must_use]
pub fn get_short_name(name: &str) -> String {
	disqualified::ShortName(name).to_string()
}
