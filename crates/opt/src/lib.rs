#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod analysis;
mod change;
mod pass;
pub mod passes;

use vmm_program::Program;

pub use self::{change::*, pass::*};

pub struct Optimizer {
	program: Program,
}
