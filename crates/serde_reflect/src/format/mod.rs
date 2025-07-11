mod float;
mod integer;

pub use self::{float::*, integer::*};

#[derive(Debug, Clone, PartialEq)]
pub enum Format {}
