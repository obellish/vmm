mod decode;
mod encode;
mod modified_utf8;
#[cfg(test)]
mod tests;

pub use self::{decode::*, encode::*};
use super::Tag;

impl Tag {
	const fn name(self) -> &'static str {
		match self {
			Self::End => "end",
			Self::Byte => "byte",
			Self::Short => "short",
			Self::Int => "int",
			Self::Long => "long",
			Self::Float => "float",
			Self::Double => "double",
			Self::ByteArray => "byte array",
			Self::String => "string",
			Self::List => "list",
			Self::Compound => "compound",
			Self::IntArray => "int array",
			Self::LongArray => "long array",
		}
	}
}
