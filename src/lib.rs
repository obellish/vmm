#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![allow(clippy::uninhabited_references)]
#![expect(unused)]

mod instr;
mod optimizer;
mod scanner;
mod unit;

pub use self::{instr::*, optimizer::*, scanner::*, unit::*};
