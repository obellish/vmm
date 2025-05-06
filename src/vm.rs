use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
};

use tracing::warn;

use super::{ExecutionUnit, Instruction, Profiler, Program, Tape, TapePointer};

pub struct Vm<R = Stdin, W = Stdout>
where
	R: Read + 'static,
	W: Write + 'static,
{
	unit: ExecutionUnit,
	input: R,
	output: W,
	profiler: Option<Profiler>,
	jump_addrs: Vec<usize>,
}

impl<R: Read, W: Write> Vm<R, W> {
	pub const fn new(unit: ExecutionUnit, input: R, output: W) -> Self {
		Self {
			unit,
			input,
			output,
			profiler: None,
			jump_addrs: Vec::new(),
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
		Vm::new(self.unit, Box::new(self.input), Box::new(self.output))
	}

	pub fn run(&mut self) -> Result<(), RuntimeError> {
		for instr in &std::mem::take(self.program_mut()) {
			self.execute_instruction(instr)?;
		}

		Ok(())
	}

	pub fn with_input<RR: Read>(self, input: RR) -> Vm<RR, W> {
		Vm::new(self.unit, input, self.output)
	}

	pub fn with_output<WW: Write>(self, output: WW) -> Vm<R, WW> {
		Vm::new(self.unit, self.input, output)
	}

	pub fn with_io<RR: Read, WW: Write>(self, input: RR, output: WW) -> Vm<RR, WW> {
		Vm::new(self.unit, input, output)
	}

	fn read_char(&mut self) -> Result<(), RuntimeError> {
		let mut buf = [0];
		if let Err(e) = self.input.read_exact(&mut buf) {
			if !matches!(e.kind(), ErrorKind::UnexpectedEof) {
				return Err(e.into());
			}
		}

		*self.unit.tape_mut().current_cell_mut() = buf[0];

		Ok(())
	}

	fn write_char(&mut self) -> Result<(), RuntimeError> {
		let ch = *self.unit.tape().current_cell();

		if ch.is_ascii() {
			self.output.write_all(&[ch])?;
		} else {
			write!(self.output, "\\0x{ch:x}")?;
		}

		self.output.flush()?;

		Ok(())
	}

	// fn current_instruction(&self) -> &Instruction {
	// 	&self.program()[self.counter]
	// }

	fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), RuntimeError> {
		if let Some(profiler) = &mut self.profiler {
			profiler.handle(instr);
		}

		match instr {
			Instruction::Add(i) => {
				*self.current_cell_mut() = self.current_cell().wrapping_add(*i as u8);
			}
			Instruction::Set(i) => *self.current_cell_mut() = *i,
			Instruction::MovePtr(i) if *i > 0 => *self.pointer_mut() += i.unsigned_abs(),
			Instruction::MovePtr(i) => *self.pointer_mut() -= i.unsigned_abs(),
			Instruction::Read => self.read_char()?,
			Instruction::Write => self.write_char()?,
			Instruction::FindZero(i) => {
				let backwards = *i < 0;
				while !matches!(self.current_cell(), 0) {
					if backwards {
						*self.pointer_mut() -= i.unsigned_abs();
					} else {
						*self.pointer_mut() += i.unsigned_abs();
					}
				}
			}
			Instruction::Loop(instructions) => {
				let mut iterations = 0usize;
				tracing::trace!(
					"entering loop at cell = {:?}, value = {}",
					self.pointer(),
					self.current_cell()
				);
				while !matches!(self.current_cell(), 0) {
					iterations += 1;

					assert!(
						(iterations <= 100_000),
						"loop exceeded 100k iterations at cell {:?}",
						self.pointer(),
					);

					for instr in instructions {
						self.execute_instruction(instr)?;
					}

					tracing::trace!("current cell value: {}", self.current_cell());
				}
			}
			_ => {}
		}

		Ok(())
	}

	const fn program(&self) -> &Program {
		self.unit.program()
	}

	const fn program_mut(&mut self) -> &mut Program {
		self.unit.program_mut()
	}

	const fn tape(&self) -> &Tape {
		self.unit.tape()
	}

	const fn tape_mut(&mut self) -> &mut Tape {
		self.unit.tape_mut()
	}

	const fn current_cell(&self) -> &u8 {
		self.tape().current_cell()
	}

	const fn current_cell_mut(&mut self) -> &mut u8 {
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
	pub fn stdio(unit: ExecutionUnit) -> Self {
		Self::new(unit, stdin(), stdout())
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
