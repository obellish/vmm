mod stack;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

pub use self::stack::Stack;
use super::{Chunk, Compiler, OpCode, Value};

const STACK_MAX: usize = 256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vm {
	current_chunk: Option<Chunk>,
	stack: Stack,
}

impl Vm {
	#[must_use]
	pub fn new() -> Self {
		Self::inner_new(None)
	}

	#[must_use]
	pub fn with_chunk(chunk: Chunk) -> Self {
		Self::inner_new(Some(chunk))
	}

	fn inner_new(current_chunk: Option<Chunk>) -> Self {
		Self {
			current_chunk,
			stack: Stack::new(),
		}
	}

	#[must_use]
	pub const fn stack(&self) -> &Stack {
		&self.stack
	}

	pub const fn stack_mut(&mut self) -> &mut Stack {
		&mut self.stack
	}

	pub fn set_chunk(&mut self, chunk: Chunk) {
		self.current_chunk.replace(chunk);
	}

	#[tracing::instrument]
	pub fn interpret(&mut self, source: String) -> InterpretResult {
		let compiler = Compiler::new(source);

		let Some(chunk) = self.current_chunk.take() else {
			return Err(InterpretError::NoChunkProvided.into());
		};

		self.run(chunk)?;

		Ok(())
	}

	#[tracing::instrument]
	fn run(&mut self, chunk: Chunk) -> InterpretResult {
		for op in chunk.iter().copied() {
			match op {
				OpCode::Constant(index) => {
					let constant = chunk.read_constant(index);
					self.stack_mut().push(constant);
				}
				OpCode::Return => {
					println!("{}", self.stack_mut().pop());
				}
				OpCode::Negate => {
					let value = self.stack_mut().pop();
					self.stack_mut().push(-value);
				}
				OpCode::Add => {
					let b = self.stack_mut().pop();
					let a = self.stack_mut().pop();
					self.stack_mut().push(a + b);
				}
				OpCode::Divide => {
					let b = self.stack_mut().pop();
					let a = self.stack_mut().pop();
					self.stack_mut().push(a / b);
				}
				OpCode::Subtract => {
					let b = self.stack_mut().pop();
					let a = self.stack_mut().pop();
					self.stack_mut().push(a - b);
				}
				OpCode::Multiply => {
					let b = self.stack_mut().pop();
					let a = self.stack_mut().pop();
					self.stack_mut().push(a * b);
				}
			}
		}

		Ok(())
	}
}

impl Default for Vm {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug)]
pub enum RuntimeError {}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for RuntimeError {}

#[derive(Debug)]
pub enum CompileError {}

impl Display for CompileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {}
	}
}

impl StdError for CompileError {}

#[derive(Debug)]
pub enum InterpretError {
	Compile(CompileError),
	Runtime(RuntimeError),
	NoChunkProvided,
}

impl Display for InterpretError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Compile(c) => Display::fmt(&c, f),
			Self::Runtime(r) => Display::fmt(&r, f),
			Self::NoChunkProvided => {
				f.write_str("no chunk was provided before attempting to interpret")
			}
		}
	}
}

impl StdError for InterpretError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Compile(c) => Some(c),
			Self::Runtime(r) => Some(r),
			Self::NoChunkProvided => None,
		}
	}
}

impl From<RuntimeError> for InterpretError {
	fn from(value: RuntimeError) -> Self {
		Self::Runtime(value)
	}
}

impl From<CompileError> for InterpretError {
	fn from(value: CompileError) -> Self {
		Self::Compile(value)
	}
}

pub type InterpretResult = std::result::Result<(), InterpretError>;
