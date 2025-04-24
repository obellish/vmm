#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod chunk;
mod compiler;
mod value;
mod vm;

use serde::{Deserialize, Serialize};

pub use self::{chunk::*, compiler::*, value::*, vm::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
	Constant(usize), // pointer to where it lives within the constant array
	Negate,
	Return,
	Add,
	Subtract,
	Multiply,
	Divide,
}
