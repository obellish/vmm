use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

use super::STACK_MAX;
use crate::Value;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stack {
	#[serde(with = "BigArray")]
	inner: [Value; STACK_MAX],
	pointer: usize,
}

impl Stack {
	#[must_use]
	pub fn new() -> Self {
		Self {
			inner: std::array::from_fn(|_| Value::default()),
			pointer: 0,
		}
	}

	pub const fn push(&mut self, value: Value) {
		self.inner[self.pointer] = value;
		self.pointer = self.pointer.saturating_add(1);
	}

	pub const fn pop(&mut self) -> Value {
		self.pointer = self.pointer.saturating_sub(1);
		self.inner[self.pointer]
	}
}

impl Default for Stack {
	fn default() -> Self {
		Self::new()
	}
}
