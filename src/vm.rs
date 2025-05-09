use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::{Error as IoError, ErrorKind, Stdin, Stdout, prelude::*, stdin, stdout},
	mem,
	num::Wrapping,
};

use super::{Instruction, Profiler, Program, Tape, TapePointer};
use crate::StackedInstruction;

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

		self.cell_mut().0 = buf[0];

		Ok(())
	}

	fn write_char(&mut self, count: usize) -> Result<(), RuntimeError> {
		let ch = self.cell().0;

		if ch.is_ascii() {
			self.output.write_all(&vec![ch; count])?;
		} else {
			let s = format!("\\0x{ch:x}").repeat(count);
			write!(self.output, "{s}")?;
		}

		self.output.flush()?;

		Ok(())
	}

	fn execute_instruction(&mut self, instr: &Instruction) -> Result<(), RuntimeError> {
		if let Some(profiler) = &mut self.profiler {
			profiler.handle(instr);
		}

		match instr {
			Instruction::Stacked(s) => self.execute_stacked_instruction(*s)?,
			Instruction::Read => self.read_char()?,
			Instruction::FindZero(i) => {
				while !matches!(self.cell().0, 0) {
					*self.pointer_mut() += *i;
				}
			}
			Instruction::RawLoop(instructions) => {
				let mut iterations = 0usize;
				while !matches!(self.cell().0, 0) {
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
			Instruction::MoveVal { offset, multiplier } => {
				let value = self.cell();
				let src_offset = self.pointer();
				let dst_offset = (*src_offset + *offset).value();
				let src_offset = src_offset.value();

				let tape = self.tape_mut();

				let src_val = mem::take(&mut tape[src_offset]);

				tape[dst_offset] += src_val.0.wrapping_mul(*multiplier);
			}
			Instruction::SetVal(i) => self.cell_mut().0 = *i,
			i => return Err(RuntimeError::Unimplemented(i.clone())),
		}

		Ok(())
	}

	fn execute_stacked_instruction(
		&mut self,
		instr: StackedInstruction,
	) -> Result<(), RuntimeError> {
		match instr {
			StackedInstruction::IncVal(i) => *self.cell_mut() += i as u8,
			StackedInstruction::MovePtr(i) => *self.pointer_mut() += i,
			StackedInstruction::Write(i) => self.write_char(i)?,
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

	fn cell(&self) -> &Wrapping<u8> {
		self.tape().current_cell()
	}

	fn cell_mut(&mut self) -> &mut Wrapping<u8> {
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
	Unimplemented(Instruction),
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => Display::fmt(&e, f),
			Self::Unimplemented(instr) => {
				f.write_str("instruction ")?;
				Debug::fmt(&instr, f)?;
				f.write_str(" is unimplemeted")
			}
		}
	}
}

impl StdError for RuntimeError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::Unimplemented(_) => None,
		}
	}
}

impl From<IoError> for RuntimeError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}
