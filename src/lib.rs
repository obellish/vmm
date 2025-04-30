#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![allow(clippy::uninhabited_references)]
#![expect(unused)]

mod instr;
mod optimizer;
pub mod passes;
mod profiler;
mod scanner;
mod unit;
mod util;
mod vm;

pub use self::{instr::*, optimizer::*, profiler::*, scanner::*, unit::*, vm::*};
