mod fmt;

use std::{
	ops::{Deref, DerefMut},
	slice,
	vec::IntoIter,
};

use serde::{Deserialize, Serialize};

use super::{OpCode, Value};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
	code: Vec<OpCode>,
	lines: Vec<usize>,
	constants: Vec<Value>,
}

impl Chunk {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			code: Vec::new(),
			lines: Vec::new(),
			constants: Vec::new(),
		}
	}

	pub fn push(&mut self, code: OpCode, line: usize) {
		self.code.push(code);
		self.lines.push(line);
	}

	pub fn push_constant(&mut self, value: Value) -> usize {
		self.constants.push(value);

		self.constants.len() - 1
	}

	#[must_use]
	pub fn read_constant(& self, idx: usize) -> Value {
		self.constants.get(idx).copied().expect("no constant found at index")
	}
}

impl Default for Chunk {
	fn default() -> Self {
		Self::new()
	}
}

impl Deref for Chunk {
	type Target = Vec<OpCode>;

	fn deref(&self) -> &Self::Target {
		&self.code
	}
}

impl DerefMut for Chunk {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.code
	}
}

impl<'a> IntoIterator for &'a Chunk {
	type IntoIter = slice::Iter<'a, OpCode>;
	type Item = &'a OpCode;

	fn into_iter(self) -> Self::IntoIter {
		self.code.iter()
	}
}

impl<'a> IntoIterator for &'a mut Chunk {
	type IntoIter = slice::IterMut<'a, OpCode>;
	type Item = &'a mut OpCode;

	fn into_iter(self) -> Self::IntoIter {
		self.code.iter_mut()
	}
}

impl IntoIterator for Chunk {
	type IntoIter = IntoIter<OpCode>;
	type Item = OpCode;

	fn into_iter(self) -> Self::IntoIter {
		self.code.into_iter()
	}
}
