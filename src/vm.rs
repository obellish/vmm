use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
};

use super::{Instruction, Profiler, Program, Tape, TapePointer};

pub struct Vm<R = Stdin, W = Stdout>
where
	R: Read + 'static,
	W: Write + 'static,
{
	program: Program,
	input: R,
	output: W,
	profiler: Option<Profiler>,
	tape: Tape,
}

impl<R: Read, W: Write> Vm<R, W> {
	pub const fn new(program: Program, input: R, output: W) -> Self {
		Self {
			program,
			input,
			output,
			profiler: None,
			tape: Tape::new(),
		}
	}

	#[must_use]
	pub const fn and_profile(mut self) -> Self {
		self.profiler = Some(Profiler::new());
		self
	}

	pub fn profiler(&self) -> Profiler {
		self.profiler.clone().unwrap_or_default()
	}

	pub fn into_dyn(self) -> Vm<Box<dyn Read>, Box<dyn Write>> {
		Vm::new(self.program, Box::new(self.input), Box::new(self.output))
	}

	pub fn run(&mut self) -> Result<(), RuntimeError> {
		for instr in &std::mem::take(self.program_mut()) {
			self.execute_instruction(instr)?;
		}

		Ok(())
	}

	pub fn with_input<RR: Read>(self, input: RR) -> Vm<RR, W> {
		Vm::new(self.program, input, self.output)
	}

	pub fn with_output<WW: Write>(self, output: WW) -> Vm<R, WW> {
		Vm::new(self.program, self.input, output)
	}

	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Vm<RR, WW> {
		Vm::new(self.program, input, output)
	}

	fn read_char(&mut self) -> Result<(), RuntimeError> {
		let mut buf = [0];
		if let Err(e) = self.input.read_exact(&mut buf) {
			if !matches!(e.kind(), ErrorKind::UnexpectedEof) {
				return Err(e.into());
			}
		}

		*self.cell_mut() = buf[0];

		Ok(())
	}

	fn write_char(&mut self) -> Result<(), RuntimeError> {
		let ch = *self.cell();

		if ch.is_ascii() {
			self.output.write_all(&[ch])?;
		} else {
			write!(self.output, "\\0x{ch:x}")?;
		}

		self.output.flush()?;

		Ok(())
	}

	fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), RuntimeError> {
		if let Some(profiler) = &mut self.profiler {
			profiler.handle(instr);
		}

		match instr {
			Instruction::Inc(i) => {
				*self.cell_mut() = self.cell().wrapping_add(*i as u8);
			}
			Instruction::Set(i) => *self.cell_mut() = *i,
			Instruction::MovePtr(i) => *self.pointer_mut() += *i,
			Instruction::Read => self.read_char()?,
			Instruction::Write => self.write_char()?,
			Instruction::FindZero(i) => {
				while !matches!(self.cell(), 0) {
					*self.pointer_mut() += *i;
				}
			}
			Instruction::Loop(instructions) => {
				let mut iterations = 0usize;
				while !matches!(self.cell(), 0) {
					iterations += 1;

					assert!(
						(iterations <= 100_000),
						"loop exceeded 100k iterations at cell {:?}",
						self.pointer(),
					);

					for instr in instructions {
						self.execute_instruction(instr)?;
					}
				}
			}
			_ => {}
		}

		Ok(())
	}

	const fn program(&self) -> &Program {
		&self.program
	}

	const fn program_mut(&mut self) -> &mut Program {
		&mut self.program
	}

	const fn tape(&self) -> &Tape {
		&self.tape
	}

	const fn tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	fn cell(&self) -> &u8 {
		self.tape().current_cell()
	}

	fn cell_mut(&mut self) -> &mut u8 {
		self.tape_mut().current_cell_mut()
	}

	const fn pointer(&self) -> &TapePointer {
		self.tape().pointer()
	}

	const fn pointer_mut(&mut self) -> &mut TapePointer {
		self.tape_mut().pointer_mut()
	}
}

impl Vm<Stdin, Stdout> {
	#[must_use]
	pub fn stdio(program: Program) -> Self {
		Self::new(program, stdin(), stdout())
	}
}

#[derive(Debug)]
pub enum RuntimeError {
	Io(IoError),
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for RuntimeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
		}
	}
}

impl From<IoError> for RuntimeError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}
