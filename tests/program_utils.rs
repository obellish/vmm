use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io,
};

use vmm::{
	interpret::{Interpreter, RuntimeError},
	opt::{NoopStore, Optimizer, OptimizerError},
	parse::{ParseError, Parser},
	program::Program,
	tape::Tape,
};

pub fn get_program(raw: &str) -> Result<Program, ParseError> {
	let raw = raw
		.chars()
		.filter(|c| matches!(c, '+'..='.' | '>' | '<' | '[' | ']'))
		.collect::<String>();

	Parser::new(&raw).scan().map(|v| v.into_iter().collect())
}

pub fn run_program<T: Tape>(program: &str, optimized: bool) -> Result<Vec<u8>, TestError> {
	let program = get_program(program)?;

	let output = {
		let program = if optimized {
			Optimizer::new(program, NoopStore::new()).optimize()?
		} else {
			program
		};

		let mut interpreter: Interpreter<T, _, _> =
			Interpreter::new(program, io::empty(), Vec::<u8>::new());

		interpreter.run()?;

		interpreter.output().clone()
	};

	Ok(output)
}

#[derive(Debug)]
pub enum TestError {
	Parse(ParseError),
	Optimizer(OptimizerError),
	Runtime(RuntimeError),
}

impl Display for TestError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Parse(e) => Display::fmt(&e, f),
			Self::Optimizer(e) => Display::fmt(&e, f),
			Self::Runtime(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for TestError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Parse(e) => Some(e),
			Self::Optimizer(e) => Some(e),
			Self::Runtime(e) => Some(e),
		}
	}
}

impl From<ParseError> for TestError {
	fn from(value: ParseError) -> Self {
		Self::Parse(value)
	}
}

impl From<OptimizerError> for TestError {
	fn from(value: OptimizerError) -> Self {
		Self::Optimizer(value)
	}
}

impl From<RuntimeError> for TestError {
	fn from(value: RuntimeError) -> Self {
		Self::Runtime(value)
	}
}

pub type Result<T, E = TestError> = std::result::Result<T, E>;
