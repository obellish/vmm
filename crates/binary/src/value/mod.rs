mod canonical;
mod integer;

use alloc::{boxed::Box, string::String, vec::Vec};

pub use self::integer::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Value {
	Integer(Integer),
	Bytes(Vec<u8>),
	Float(f64),
	Text(String),
	Bool(bool),
	Null,
	Tag(u64, Box<Self>),
	Array(Vec<Self>),
	Map(Vec<(Self, Self)>),
}
