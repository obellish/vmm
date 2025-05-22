#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod profiler;

use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind as IoErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
	mem,
	num::{NonZero, Wrapping},
};

use vmm_ir::{Instruction, Offset};
use vmm_program::Program;
use vmm_tape::{Tape, TapePointer};

pub use self::profiler::*;

pub const ITERATION_LIMIT: usize = 100_000;

#[derive(Debug, Clone)]
pub struct Interpreter<R = Stdin, W = Stdout> {
	program: Program,
	input: R,
	output: W,
	profiler: Option<Profiler>,
	tape: Tape,
}

impl<R, W> Interpreter<R, W> {
	pub const fn new(program: Program, input: R, output: W) -> Self {
		Self {
			program,
			input,
			output,
			profiler: None,
			tape: Tape::new(),
		}
	}

	pub const fn with_profiler(program: Program, input: R, output: W) -> Self {
		Self::new(program, input, output).and_with_profiler()
	}

	#[must_use]
	pub const fn and_with_profiler(mut self) -> Self {
		self.profiler = Some(Profiler::new());
		self
	}

	pub fn profiler(&self) -> Profiler {
		self.profiler.unwrap_or_default()
	}

	pub const fn program(&self) -> &Program {
		&self.program
	}

	pub const fn program_mut(&mut self) -> &mut Program {
		&mut self.program
	}

	pub const fn tape(&self) -> &Tape {
		&self.tape
	}

	pub const fn tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	pub fn cell(&self) -> &Wrapping<u8> {
		self.tape().cell()
	}

	pub fn cell_mut(&mut self) -> &mut Wrapping<u8> {
		self.tape_mut().cell_mut()
	}

	pub const fn ptr(&self) -> &TapePointer {
		self.tape().ptr()
	}

	pub const fn ptr_mut(&mut self) -> &mut TapePointer {
		self.tape_mut().ptr_mut()
	}
}

impl<R, W> Interpreter<R, W>
where
	R: Read + 'static,
	W: Write + 'static,
{
	pub fn into_dyn(self) -> Interpreter<Box<dyn Read>, Box<dyn Write>> {
		Interpreter::new(self.program, Box::new(self.input), Box::new(self.output))
	}

	pub fn with_input<RR: Read>(self, input: RR) -> Interpreter<RR, W> {
		Interpreter::new(self.program, input, self.output)
	}

	pub fn with_output<WW: Write>(self, output: WW) -> Interpreter<R, WW> {
		Interpreter::new(self.program, self.input, output)
	}

	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Interpreter<RR, WW> {
		Interpreter::new(self.program, input, output)
	}

	pub fn run(&mut self) -> Result<(), RuntimeError> {
		for instr in &std::mem::take(self.program_mut()) {
			self.execute_instruction(instr)?;
		}

		Ok(())
	}

	fn read_char(&mut self) -> Result<(), RuntimeError> {
		loop {
			let mut buf = [0];
			let err = self.input.read_exact(&mut buf);
			match err.as_ref().map_err(IoError::kind) {
				Err(IoErrorKind::UnexpectedEof) => {
					mem::take(&mut buf);
				}
				_ => err?,
			}

			if cfg!(target_os = "windows") && matches!(buf[0], b'\r') {
				continue;
			}

			break;
		}
		Ok(())
	}

	fn write_char(&mut self) -> Result<(), RuntimeError> {
		let ch = self.cell().0;

		if !cfg!(target_os = "windows") || ch < 128 {
			self.output.write_all(&[ch])?;
			self.output.flush()?;
		}

		Ok(())
	}

	fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), RuntimeError> {
		if let Some(profiler) = &mut self.profiler {
			profiler.handle(instr);
		}

		match instr {
			Instruction::IncVal {
				value: i,
				offset: None,
			} => *self.cell_mut() += *i as u8,
			Instruction::SetVal {
				value: i,
				offset: None,
			} => self.cell_mut().0 = i.map_or(0, NonZero::get),
			Instruction::MovePtr(Offset::Relative(i)) => *self.ptr_mut() += *i,
			Instruction::Write => self.write_char()?,
			Instruction::Read => self.read_char()?,
			Instruction::FindZero(i) => {
				while !matches!(self.cell().0, 0) {
					*self.ptr_mut() += *i;
				}
			}
			Instruction::DynamicLoop(instructions) => {
				let mut iterations = 0usize;
				while !matches!(self.cell().0, 0) {
					iterations += 1;

					if matches!(iterations, ITERATION_LIMIT) {
						return Err(RuntimeError::TooManyIterations(self.ptr().value()));
					}

					for instr in instructions {
						self.execute_instruction(instr)?;
					}
				}
			}
			Instruction::MoveVal {
				offset: Offset::Relative(offset),
				factor: multiplier,
			} => {
				let (src_offset, dst_offset) = {
					let src_offset = self.ptr();
					(src_offset.value(), (*src_offset + *offset).value())
				};

				let tape = self.tape_mut();

				let src_val = mem::take(&mut tape[src_offset]);

				tape[dst_offset] += src_val.0.wrapping_mul(*multiplier);
			}
			Instruction::IncVal {
				value,
				offset: Some(Offset::Relative(x)),
			} => {
				let dst_offset = (*self.ptr() + *x).value();

				let tape = self.tape_mut();

				tape[dst_offset] += *value as u8;
			}
			Instruction::SetVal {
				value,
				offset: Some(Offset::Relative(x)),
			} => {
				let dst_offset = (*self.ptr() + *x).value();

				let tape = self.tape_mut();

				tape[dst_offset].0 = value.map_or(0, NonZero::get);
			}
			i => return Err(RuntimeError::Unimplemented(i.clone())),
		}

		Ok(())
	}
}

impl Interpreter<Stdin, Stdout> {
	#[must_use]
	pub fn stdio(program: Program) -> Self {
		Self::new(program, stdin(), stdout())
	}
}

#[derive(Debug)]
pub enum RuntimeError {
	Io(IoError),
	Unimplemented(Instruction),
	TooManyIterations(usize),
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
			Self::Unimplemented(instr) => {
				f.write_str("instruction ")?;
				Debug::fmt(&instr, f)?;
				f.write_str(" is unimplmented")
			}
			Self::TooManyIterations(i) => {
				f.write_str("loop exceeded ")?;
				Display::fmt(&ITERATION_LIMIT, f)?;
				f.write_str(" iterations at cell ")?;
				Display::fmt(&i, f)
			}
		}
	}
}

impl StdError for RuntimeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::Unimplemented(_) | Self::TooManyIterations(_) => None,
		}
	}
}

impl From<IoError> for RuntimeError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}
